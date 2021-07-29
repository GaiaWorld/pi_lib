//! 资源管理器， 管理多个资源表。有最大内存容量的控制。
//! 一个资源可能会用于不同的用途， 尤其是图片， 界面、人物、场景、特效等， 每种用途的优先级不同。
//! 为了更好的缓存资源，因此我们将不同的用途定义为分组， 为每类每用途的资源创建资源表。
//! 需要外部设置的每个资源表内的每分组Cache的max_capacity和min_capacity及超时时间。
//! 如果总容量有空闲， 则按权重提高那些满的Cache的cur_capacity

use hash::XHashMap;
use share::Share;
use std::any::TypeId;
use std::collections::hash_map::Entry;

use crate::res_map::{Res, ResCollect, ResMap};

pub static CAPACITY: usize = 256 * 1024 * 1024;

pub static MIN: usize = 1024 * 1024;
pub static MAX: usize = 4 * 1024 * 1024;
pub static TIMEOUT: usize = 8 * 60 * 1000;

/// 资源管理器
pub struct ResMgr {
    /// 资源类型表
    tables: XHashMap<TypeId, Box<dyn ResCollect>>,
    /// 最大内存容量
    total_capacity: usize,
    /// 统计每个资源分组缓存队列的最小容量
    min_capacity: usize,
    /// 统计每个资源分组缓存队列的权重（最大容量 - 最小容量）
    weight: usize,
    /// 整理时用的临时数组，超过预期容量的Cache
    temp: Vec<(TypeId, usize, usize)>,
}

impl Default for ResMgr {
    fn default() -> Self {
        ResMgr::with_capacity(CAPACITY)
    }
}

impl ResMgr {
    /// 用指定的最大内存容量创建资源管理器
    pub fn with_capacity(total_capacity: usize) -> Self {
        ResMgr {
            tables: Default::default(),
            total_capacity,
            min_capacity: 0,
            weight: 1,
            temp: Vec::new(),
        }
    }
    /// 获得总内存容量
    pub fn total_capacity(&self) -> usize {
        self.total_capacity
    }
    /// 获得全部资源缓存的累计最小内存容量
    pub fn min_capacity(&self) -> usize {
        self.min_capacity
    }
    /// 获得资源管理器的内存容量
    pub fn mem_size(&self) -> usize {
        let mut r = 0;
        for v in self.tables.values() {
            r += v.size();
        }
        r
    }

    /// 注册指定类型的资源表。 参数为资源表的指定分组的配置。
    #[inline]
    pub fn register<T: Res + 'static>(
        &mut self,
        group_i: usize,
        mut min_capacity: usize,
        max_capacity: usize,
        timeout: usize,
        name: Share<String>,
    ) {
        if min_capacity > max_capacity {
            min_capacity = max_capacity;
        }
        self.weight += max_capacity - min_capacity;
        self.min_capacity += min_capacity;
        match self.tables.entry(TypeId::of::<T>()) {
            Entry::Occupied(mut e) => {
                let c = &mut e.get_mut().caches()[group_i];
                self.weight -= c.max_capacity - c.min_capacity;
                self.min_capacity -= c.min_capacity;
                c.config(min_capacity, max_capacity, timeout, name.clone());
            }
            Entry::Vacant(e) => {
                let mut map = Box::new(ResMap::<T>::default());
                let c = &mut map.caches()[group_i];
                c.config(min_capacity, max_capacity, timeout, name);
                e.insert(map);
            }
        }
        if self.total_capacity > self.min_capacity {
            return;
        }
        // 如果设置的总容量小于累计最小容量，则所有资源缓存队列的当前容量都为最小容量
        for v in self.tables.values_mut() {
            let arr = v.caches();
            for c in arr {
                c.cur_capacity = c.min_capacity;
            }
        }
        // println!("----------total: {}, min: {}, weight: {}", self.total_capacity, self.min_capacity, self.weight);
    }
    /// 获取指定类型<T>的资源表
    pub fn fetch_map<T: Res + 'static>(&self) -> Option<&ResMap<T>> {
        match self.tables.get(&TypeId::of::<T>()) {
            Some(r) => <dyn ResCollect>::downcast_ref::<ResMap<T>>(r.as_ref()),
            _ => None,
        }
    }
    /// 获取指定类型<T>的资源表
    pub fn fetch_map_mut<T: Res + 'static>(&mut self) -> Option<&mut ResMap<T>> {
        match self.tables.get_mut(&TypeId::of::<T>()) {
            Some(r) => <dyn ResCollect>::downcast_mut::<ResMap<T>>(r.as_mut()),
            _ => None,
        }
    }
    /// 获得指定键的资源
    pub fn get<T: Res + 'static>(&mut self, key: &<T as Res>::Key) -> Option<Share<T>> {
        match self.tables.get_mut(&TypeId::of::<T>()) {
            Some(r) => match <dyn ResCollect>::downcast_mut::<ResMap<T>>(r.as_mut()) {
                Some(map) => map.get(key),
                _ => None,
            },
            _ => None,
        }
    }
    /// 放入资源
    #[inline]
    pub fn insert<T: Res + 'static>(&mut self, res: Share<T>) {
        match self.tables.get_mut(&TypeId::of::<T>()) {
            Some(r) => match <dyn ResCollect>::downcast_mut::<ResMap<T>>(r.as_mut()) {
                Some(map) => map.insert(res),
                _ => panic!("downcast error!"),
            },
            None => panic!("TypeId not found!"),
        }
    }
    /// 移除一个指定键的资源
    #[inline]
    pub fn remove<T: Res + 'static>(&mut self, key: &<T as Res>::Key) -> Option<Share<T>> {
        match self.tables.get_mut(&TypeId::of::<T>()) {
            Some(r) => match <dyn ResCollect>::downcast_mut::<ResMap<T>>(r.as_mut()) {
                Some(map) => map.remove(key),
                _ => None,
            },
            _ => None,
        }
    }
    /// 整理方法， 将无人使用的资源放入到Cache， 清理超时的资源。Mgr有总内存容量， 按权重分给其下的Cache。 如果总容量有空闲， 则按权重提高那些满的Cache的cur_capacity
    pub fn collect(&mut self, now: usize) {
        // 最小容量下，仅进行最小容量清理操作
        if self.total_capacity <= self.min_capacity {
            for v in self.tables.values_mut() {
                v.timeout_collect(now);
                v.garbage(now);
                v.capacity_collect();
            }
            return;
        }
        let weight_capacity = (self.total_capacity - self.min_capacity) as f32;
        let mut overflow = 0; // 超过权重的容量， 溢出大小
        let mut free_size = 0; // 超过权重的容量，空闲容量
                               // 先用超时整理腾出空间，统计每种资源的占用
        for (k, v) in self.tables.iter_mut() {
            v.timeout_collect(now);
            v.garbage(now);
            let arr = v.caches();
            for i in 0..arr.len() {
                let c = &mut arr[i];
                if c.max_capacity <= c.min_capacity {
                    continue;
                }
                // 该缓存队列的权重占比
                let weight = (c.max_capacity - c.min_capacity) as f32 / (self.weight as f32);
                // 预期容量，该cache根据权重算出来的可增加的内存容量，加上min_capacity
                let capacity = (weight_capacity * weight) as usize + c.min_capacity;
                // println!("{}, ccc: size:{} cur:{} min:{} max:{}, capacity:{} weight:{}", c.name, c.size, c.cur_capacity, c.min_capacity, c.max_capacity, capacity, weight);
                // 如果当前内存小于预期容量
                if c.size <= capacity {
                    // 将当前容量设置为预期容量
                    c.cur_capacity = capacity;
                    // 将多余容量放到free_size上
                    free_size += capacity - c.size;
                } else {
                    // 如果当前内存大于预期容量
                    // 将超出大小放到overflow上
                    overflow += c.size - capacity;
                    self.temp.push((k.clone(), i, capacity));
                }
            }
        }
        //println!("----------free_size: {}, overflow: {}, temp: {}", free_size, overflow, self.temp.len());
        // 将空闲大小， 按超出比例，调整溢出Cache的当前容量
        if self.temp.len() > 0 {
            if free_size > 0 {
                let free = free_size as f32;
                let overflow = overflow as f32;
                for (tid, index, capacity) in &self.temp {
                    let map = self.tables.get_mut(&tid).unwrap();
                    let c = &mut map.caches()[*index];

                    c.cur_capacity =
                        *capacity + ((c.size - *capacity) as f32 * free / overflow) as usize;
                    //println!("{}, ccc: size:{} cur:{} min:{} max:{}, capacity:{}", c.name, c.size, c.cur_capacity, c.min_capacity, c.max_capacity, capacity);
                }
            }
            self.temp.clear();
        }
        // 超量整理
        for (_, v) in self.tables.iter_mut() {
            v.capacity_collect();
        }
    }
}

// 测试定时器得延时情况
#[cfg(test)]
mod test_mod {

    use share::Share;

    //use self::rand_core::{RngCore, SeedableRng};
    use crate::*;

    #[cfg(test)]
    struct R1(usize, usize, usize);

    #[cfg(test)]
    impl Res for R1 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
        /// 所在的分组
        fn group(&self) -> usize {
            self.2
        }
    }

    #[cfg(test)]
    struct R2(usize, usize);

    #[cfg(test)]
    impl Res for R2 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
    }

    #[cfg(test)]
    struct R3(usize, usize);

    #[cfg(test)]
    impl Res for R3 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
    }
    #[cfg(test)]
    struct R4(usize, usize);

    #[cfg(test)]
    impl Res for R4 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
    }

    #[cfg(test)]
    struct R5(usize, usize);

    #[cfg(test)]
    impl Res for R5 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
    }

    #[cfg(test)]
    struct R6(usize, usize);

    #[cfg(test)]
    impl Res for R6 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
    }
    #[cfg(test)]
    struct R7(usize, usize);

    #[cfg(test)]
    impl Res for R7 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
    }

    #[cfg(test)]
    struct R8(usize, usize);

    #[cfg(test)]
    impl Res for R8 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
    }

    #[cfg(test)]
    struct R9(usize, usize);

    #[cfg(test)]
    impl Res for R9 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
    }
    #[cfg(test)]
    struct R10(usize, usize);

    #[cfg(test)]
    impl Res for R10 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
    }

    #[cfg(test)]
    struct R11(usize, usize);

    #[cfg(test)]
    impl Res for R11 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
    }

    #[cfg(test)]
    struct R12(usize, usize);

    #[cfg(test)]
    impl Res for R12 {
        type Key = usize;
        /// 资源的关联键
        fn key(&self) -> Self::Key {
            self.0
        }
        /// 资源的大小
        fn size(&self) -> usize {
            self.1
        }
    }

    #[cfg(test)]
    pub fn create_res_mgr(total_capacity: usize) -> ResMgr {
        let mut res_mgr = if total_capacity > 0 {
            ResMgr::with_capacity(total_capacity)
        } else {
            ResMgr::default()
        };

        res_mgr.register::<R1>(
            0,
            10 * 1024 * 1024,
            50 * 1024 * 1024,
            5 * 60000,
            Share::new("TextureRes".to_string()),
        );
        res_mgr.register::<R2>(
            0,
            20 * 1024,
            100 * 1024,
            5 * 60000,
            Share::new("GeometryRes".to_string()),
        );
        res_mgr.register::<R3>(
            0,
            20 * 1024,
            100 * 1024,
            5 * 60000,
            Share::new("BufferRes".to_string()),
        );

        res_mgr.register::<R4>(
            0,
            512,
            1024,
            60 * 60000,
            Share::new("SamplerRes".to_string()),
        );
        res_mgr.register::<R5>(
            0,
            512,
            1024,
            60 * 60000,
            Share::new("RasterStateRes".to_string()),
        );
        res_mgr.register::<R6>(
            0,
            512,
            1024,
            60 * 60000,
            Share::new("BlendStateRes".to_string()),
        );
        res_mgr.register::<R7>(
            0,
            512,
            1024,
            60 * 60000,
            Share::new("StencilStateRes".to_string()),
        );
        res_mgr.register::<R8>(
            0,
            512,
            1024,
            60 * 60000,
            Share::new("DepthStateRes".to_string()),
        );

        res_mgr.register::<R9>(
            0,
            4 * 1024,
            8 * 1024,
            60 * 60000,
            Share::new("UColorUbo".to_string()),
        );
        res_mgr.register::<R10>(
            0,
            1 * 1024,
            2 * 1024,
            60 * 60000,
            Share::new("HsvUbo".to_string()),
        );
        res_mgr.register::<R11>(
            0,
            1 * 1024,
            2 * 1024,
            60 * 60000,
            Share::new("MsdfStrokeUbo".to_string()),
        );
        res_mgr.register::<R12>(
            0,
            1 * 1024,
            2 * 1024,
            60 * 60000,
            Share::new("CanvasTextStrokeColorUbo".to_string()),
        );
        res_mgr
    }

    #[test]
    pub fn test() {
        let total_capacity: usize = 16008864;
        let mut res_mgr = create_res_mgr(total_capacity);
        res_mgr.insert::<R1>(Share::new(R1(123456789, 262144, 0)));
        res_mgr.insert::<R3>(Share::new(R3(3902250154, 32)));
        res_mgr.insert::<R3>(Share::new(R3(1519695964, 16)));
        res_mgr.insert::<R6>(Share::new(R6(2905594028, 8)));
        res_mgr.insert::<R6>(Share::new(R6(3006512311, 8)));
        res_mgr.insert::<R4>(Share::new(R4(308248423, 8)));
        res_mgr.insert::<R4>(Share::new(R4(2591543091, 8)));
        res_mgr.insert::<R9>(Share::new(R9(2106312588, 8)));
        res_mgr.insert::<R9>(Share::new(R9(796366362, 8)));
        res_mgr.insert::<R11>(Share::new(R11(3879787636, 8)));
        res_mgr.insert::<R12>(Share::new(R12(1145791972, 8)));
        println!(
            "xxxxxxxxxxxxxxxxxxxxxxxxxx1: mem:{:?} total:{:?} min:{:?}",
            res_mgr.mem_size(),
            res_mgr.total_capacity(),
            res_mgr.min_capacity(),
        );
        res_mgr.collect(0);
        println!("xxxxxxxxxxxxxxxxxxxxxxxxxx2: {:?}", res_mgr.mem_size());
        res_mgr.insert::<R9>(Share::new(R9(2492188942, 8)));
        res_mgr.insert::<R9>(Share::new(R9(4246038113, 8)));
        res_mgr.insert::<R9>(Share::new(R9(2564464371, 8)));
        res_mgr.insert::<R9>(Share::new(R9(1601737751, 8)));

        res_mgr.insert::<R1>(Share::new(R1(1, 131072, 0)));
        res_mgr.insert::<R1>(Share::new(R1(2, 65536, 0)));
        res_mgr.insert::<R1>(Share::new(R1(3, 1572864, 0)));

        res_mgr.insert::<R3>(Share::new(R3(1156469915, 32)));
        res_mgr.insert::<R2>(Share::new(R2(940515885, 32)));
        res_mgr.insert::<R3>(Share::new(R3(2294013149, 32)));
        res_mgr.insert::<R2>(Share::new(R2(1197965350, 32)));
        res_mgr.insert::<R3>(Share::new(R3(2418548769, 32)));
        res_mgr.insert::<R2>(Share::new(R2(203047359, 32)));

        res_mgr.insert::<R1>(Share::new(R1(4, 4194304, 0)));
        res_mgr.insert::<R1>(Share::new(R1(5, 4194304, 0)));
        res_mgr.insert::<R1>(Share::new(R1(6, 4194304, 0)));
        res_mgr.insert::<R1>(Share::new(R1(7, 4194304, 0)));
        res_mgr.insert::<R1>(Share::new(R1(8, 4194304, 0)));
        res_mgr.insert::<R1>(Share::new(R1(9, 4194304, 0)));

        // buffer.create(2402797892, R3 {}, 32, 0);
        // geometry_res.create(1733445847, R2 {}, 0, 0);
        // texture.create(Atom::from("4"), R1 {}, 4194304, 0);
        // texture.create(Atom::from("5"), R1 {}, 4194304, 0);

        // buffer.create(1, R3 {}, 32, 0);
        // geometry_res.create(1, R2 {}, 0, 0);
        // buffer.create(2, R3 {}, 32, 0);
        // geometry_res.create(2, R2 {}, 0, 0);
        // res_mgr.collect(2000);

        // texture.create(Atom::from("6"), R1 {}, 1048576, 0);

        // buffer.create(3, R3 {}, 32, 0);
        // geometry_res.create(3, R2 {}, 0, 0);
        // buffer.create(4, R3 {}, 32, 0);
        // geometry_res.create(4, R2 {}, 0, 0);
        // u_color_ubo.create(1009613414, R9 {}, 0, 0);
        // buffer.create(5, R3 {}, 32, 0);
        // geometry_res.create(5, R2 {}, 0, 0);
        // buffer.create(6, R3 {}, 32, 0);
        // geometry_res.create(6, R2 {}, 0, 0);
        // buffer.create(7, R3 {}, 32, 0);
        // geometry_res.create(7, R2 {}, 0, 0);
        // buffer.create(8, R3 {}, 32, 0);
        // geometry_res.create(8, R2 {}, 0, 0);
        // buffer.create(9, R3 {}, 32, 0);
        // geometry_res.create(9, R2 {}, 0, 0);
        // u_color_ubo.create(4173812235, R9 {}, 0, 0);
        // texture.create(Atom::from("7"), R1 {}, 1572864, 0);
        // texture.create(Atom::from("8"), R1 {}, 65536, 0);
        // buffer.create(10, R3 {}, 32, 0);
        // geometry_res.create(10, R2 {}, 0, 0);
        // texture.create(Atom::from("16"), R1 {}, 1048576, 0);
        // texture.create(Atom::from("9"), R1 {}, 2097152, 0);
        // texture.create(Atom::from("10"), R1 {}, 1048576, 0);
        // texture.create(Atom::from("11"), R1 {}, 2097152, 0);
        // texture.create(Atom::from("12"), R1 {}, 2097152, 0);
        // buffer.create(11, R3 {}, 32, 0);
        // geometry_res.create(11, R2 {}, 0, 0);
        // buffer.create(12, R3 {}, 32, 0);
        // geometry_res.create(12, R2 {}, 0, 0);
        // buffer.create(13, R3 {}, 32, 0);
        // geometry_res.create(13, R2 {}, 0, 0);
        // buffer.create(14, R3 {}, 32, 0);
        // geometry_res.create(14, R2 {}, 0, 0);

        // texture.create(Atom::from("13"), R1 {}, 2097152, 0);
        println!("xxxxxxxxxxxxxxxxxxxxxxxxxx3: {:?}", res_mgr.mem_size());
        res_mgr.collect(3000);
        println!("xxxxxxxxxxxxxxxxxxxxxxxxxx4: {:?}", res_mgr.mem_size());

        // let r1 = res_mgr.fetch_map().unwrap();
    }
}
