// 资源管理器， 管理多个资源表。有总内存上限的控制。
// 初始化时，设置的每个资源表内的LRU的max_capacity和min_capacity的差，就是每个LRU的权重。
// 如果有LRU有空闲， 则会减少其max_capacity, 按权重提高那些满的LRU的max_capacity

use std::any::TypeId;

use hash::XHashMap;
use share::Share;

use super::res_map::{Res, ResCollect, ResMap, StateInfo};

pub static CAPACITY: usize = 16 * 1024 * 1024;

/// 资源管理器
pub struct ResMgr {
    tables: XHashMap<(TypeId, usize/*group_i*/), ResTable>,
    pub total_capacity: usize,
    weight: usize,
    min_capacity: usize,
}

struct ResTable{
    res_map: Share<dyn ResCollect>,
    weight: usize,
}

impl Default for ResMgr {
    fn default() -> Self {
        ResMgr::with_capacity(CAPACITY)
    }
}

impl ResMgr {
    pub fn with_capacity(total_capacity: usize) -> Self {
        ResMgr {
            tables: XHashMap::default(),
            total_capacity,
            weight: 0,
            min_capacity: 0,
        }
    }

    pub fn mem_size(&self) -> usize {
        0
        // let mut r = 0;
        // for (_, v) in self.tables.iter() {
        //     r += v.0.mem_size();
        // }
        // r
    }

    pub fn info(&self) -> usize {
        let mut r = 0;
        for (_, v) in self.tables.iter() {
            r += v.res_map.mem_size();
        }
        r
    }

    /// 注册指定类型的资源表。 参数为资源表的3种lru的配置。 [min_capacity1, max_capacity1, timeout1, min_capacity2, max_capacity2, timeout2, min_capacity3, max_capacity3, timeout3]。 如果不使用后2种，直接将min_capacity, max_capacity都设成0。
    #[inline]
    pub fn register<T: Res + 'static>(&mut self, min_capacity: usize, max_capacity: usize, timeout: usize, group_i: usize, name: String) {
        let weight = max_capacity - min_capacity; // 权重
        self.weight += weight;
        self.min_capacity += min_capacity;
        let new_weight = &mut self.weight;
        let new_min_capacity = &mut self.min_capacity;
        self.tables
            .entry((TypeId::of::<T>(), group_i))
            .and_modify(|e| {
                let r = match e.res_map.clone().downcast::<ResMap<T>>() {
                    Ok(r) => r,
                    Err(_) => return,
                };
                let old = get_mut(&*(r)).modify_config(min_capacity, max_capacity, timeout);
                *new_weight -= e.weight;
                e.weight = weight;
                *new_min_capacity -= old.0;
            })
            .or_insert(ResTable{res_map: Share::new(ResMap::<T>::with_config(name, min_capacity, max_capacity, timeout)), weight});
    }

    pub fn fetch_map<T: Res>(&self, group_i: usize) -> Option<Share<ResMap<T>>> {
        match self.tables.get(&(TypeId::of::<T>(), group_i)) {
            Some(i) => match i.res_map.clone().downcast::<ResMap<T>>() {
                Ok(r) => Some(r),
                Err(_) => None,
            },
            _ => None,
        }
    }

    pub fn get<T: Res + 'static>(&self, name: &<T as Res>::Key, group_i: usize) -> Option<Share<T>> {
        match self.tables.get(&(TypeId::of::<T>(), group_i)) {
            Some(i) => match i.res_map.clone().downcast::<ResMap<T>>() {
                Ok(r) => match get_mut(&*r).get(name) {
                    Some(r) => Some(r),
                    None => None,
                },
                Err(_) => None,
            },
            _ => None,
        }
    }

    #[inline]
    pub fn create<T: Res + 'static>(
        &mut self,
		name: T::Key,
		group_i: usize,
        value: T,
        cost: usize,
    ) -> Share<T> {
        match self.tables.get(&(TypeId::of::<T>(), group_i)) {
            Some(i) => match i.res_map.clone().downcast::<ResMap<T>>() {
                Ok(r) => get_mut(&*r).create(name, value, cost, group_i),
                Err(_) => panic!("downcast error!"),
            },
            None => panic!("TypeId not found!"),
        }
    }

    #[inline]
    pub fn remove<T: Res + 'static>(&mut self, name: &<T as Res>::Key, group_i: usize) -> Option<Share<T>> {
        match self.tables.get(&(TypeId::of::<T>(), group_i)) {
            Some(i) => match i.res_map.clone().downcast::<ResMap<T>>() {
                Ok(r) => get_mut(&*r).remove(name),
                Err(_) => None,
            },
            _ => None,
        }
    }
    // 整理方法， 将无人使用的资源放入到LruCache， 清理过时的资源
    // 就是LruMgr有总内存上限， 按权重分给其下的LRU。 如果有LRU有空闲， 则会减少其max_size, 按权重提高那些满的LRU的max_size
    pub fn collect(&mut self, now: usize) {
        let capacity = self.total_capacity as isize - self.min_capacity as isize;
        // println!(
        //     "resmgr collect1============now: {}, total_capacity:{}, min_capacity:{}, capacity:{}",
        //     now, self.total_capacity as isize, self.min_capacity as isize, capacity
        // );
        let capacity = if capacity < 0 { 0 } else { capacity as usize };
        let mut up_size = 0; // 超过权重的总大小
        let mut down_size = 0; // 小于权重的总大小
        let mut vec = Vec::new(); // map引用
        let mut up_full = Vec::new(); // 超过权重并满了的map_index
        let mut up_ok = Vec::new(); // 超过权重并Ok的map_index

        for v in self.tables.values() {
            let vm = &*(v.res_map);
            let map = unsafe { &mut *(vm as *const dyn ResCollect as *mut dyn ResCollect) };
            let state_info = map.collect(now);
            let calc_max = if self.weight == 0 {
                0
            } else {
                (capacity as f32 * v.weight as f32 / self.weight as f32) as usize
            };
            // let calc_max = (capacity as f32 * v.1[i] as f32 / self.weight as f32) as usize; // 该lru根据权重算出来的可增加的内存总量，如果加上min_capacity则是最大容量max_capacity
            match state_info {
                StateInfo::Full(min, size) => {
                    // 如果当前大小小于权重大小，则扩大容量到权重大小
                    if size < min + calc_max {
                        map.set_max_capacity(min + calc_max);
                    } else if size > min + calc_max {
                        up_size += size - (min + calc_max);
                        up_full.push((vec.len(), size));
                    }
                }
                StateInfo::Ok(min, size) => {
                    if size < min + calc_max {
                        down_size += min + calc_max - size;
                    } else if size > min + calc_max {
                        up_size += size - (min + calc_max);
                        up_ok.push((vec.len(), size));
                    }
                }
                StateInfo::Free(min, _size, right_size) => {
                    map.set_max_capacity(right_size);
                    if right_size < min + calc_max {
                        down_size += min + calc_max - right_size;
                    } else {
                        up_size += right_size - (min + calc_max);
                        up_ok.push((vec.len(), right_size));
                    }
                }
                _ => (),
            }
            // for ss in arr.iter() {
                
            //     i += 1;
            // }
            vec.push(map);
        }
        // println!(
        //     "resmgr collect2============up_size: {}, down_size:{}, up_full: {:?}, up_ok:{:?}",
        //     up_size, down_size, &up_full, &up_ok
        // );
        if up_size > down_size && up_full.len() + up_ok.len() > 0 {
            // 如果超过的权重比小于的权重大，表示需要控制大小，将up_full和up_ok的lru的容量变小，
            let del = (up_size - down_size) / (up_full.len() + up_ok.len());
            for v in up_full {
                let map = unsafe { vec.get_unchecked_mut(v.0) };
                map.set_max_capacity( if v.1 > del { v.1 - del } else { 0 });
            }
            for v in up_ok {
                let map = unsafe { vec.get_unchecked_mut(v.0) };
                map.set_max_capacity( if v.1 > del { v.1 - del } else { 0 });
            }
        } else if up_size < down_size && up_full.len() > 0 {
            // 表示有空闲大小， 将up_full的lru的容量扩大
            let add = (down_size - up_size) / up_full.len();
            for v in up_full {
                let map = unsafe { vec.get_unchecked_mut(v.0) };
                map.set_max_capacity( v.1 + add);
            }
        }

        for m in vec {
            m.capacity_collect();
        }
    }
}

fn get_mut<T: Res>(map: &ResMap<T>) -> &mut ResMap<T> {
    unsafe { &mut *(map as *const ResMap<T> as *mut ResMap<T>) }
}

#[cfg(test)]
extern crate atom;
#[cfg(test)]
use self::atom::Atom;
#[cfg(test)]
struct R1 {}

#[cfg(test)]
impl Res for R1 {
    type Key = Atom;
}

#[cfg(test)]
struct R2 {}

#[cfg(test)]
impl Res for R2 {
    type Key = usize;
}

#[cfg(test)]
struct R3 {}

#[cfg(test)]
impl Res for R3 {
    type Key = usize;
}
#[cfg(test)]
struct R4 {}

#[cfg(test)]
impl Res for R4 {
    type Key = usize;
}

#[cfg(test)]
struct R5 {}

#[cfg(test)]
impl Res for R5 {
    type Key = usize;
}

#[cfg(test)]
struct R6 {}

#[cfg(test)]
impl Res for R6 {
    type Key = usize;
}
#[cfg(test)]
struct R7 {}

#[cfg(test)]
impl Res for R7 {
    type Key = usize;
}

#[cfg(test)]
struct R8 {}

#[cfg(test)]
impl Res for R8 {
    type Key = usize;
}

#[cfg(test)]
struct R9 {}

#[cfg(test)]
impl Res for R9 {
    type Key = usize;
}
#[cfg(test)]
struct R10 {}

#[cfg(test)]
impl Res for R10 {
    type Key = usize;
}

#[cfg(test)]
struct R11 {}

#[cfg(test)]
impl Res for R11 {
    type Key = usize;
}

#[cfg(test)]
struct R12 {}

#[cfg(test)]
impl Res for R12 {
    type Key = usize;
}

#[cfg(test)]
pub fn create_res_mgr(total_capacity: usize) -> ResMgr {
    let mut res_mgr = if total_capacity > 0 {
        ResMgr::with_capacity(total_capacity)
    } else {
        ResMgr::default()
    };

    res_mgr.register::<R1>(
        10 * 1024 * 1024,
        50 * 1024 * 1024,
        5 * 60000,
        0,
        "TextureRes".to_string(),
    );
    res_mgr.register::<R2>(
        20 * 1024, 100 * 1024, 5 * 60000, 0,
        "GeometryRes".to_string(),
    );
    res_mgr.register::<R3>(
        20 * 1024, 100 * 1024, 5 * 60000, 0,
        "BufferRes".to_string(),
    );

    res_mgr.register::<R4>(
        512, 1024, 60 * 60000, 0,
        "SamplerRes".to_string(),
    );
    res_mgr.register::<R5>(
        512, 1024, 60 * 60000, 0,
        "RasterStateRes".to_string(),
    );
    res_mgr.register::<R6>(
        512, 1024, 60 * 60000, 0,
        "BlendStateRes".to_string(),
    );
    res_mgr.register::<R7>(
        512, 1024, 60 * 60000, 0,
        "StencilStateRes".to_string(),
    );
    res_mgr.register::<R8>(
        512, 1024, 60 * 60000, 0,
        "DepthStateRes".to_string(),
    );

    res_mgr.register::<R9>(
        4 * 1024, 8 * 1024, 60 * 60000, 0,
        "UColorUbo".to_string(),
    );
    res_mgr.register::<R10>(
        1 * 1024, 2 * 1024, 60 * 60000, 0,
        "HsvUbo".to_string(),
    );
    res_mgr.register::<R11>(
        1 * 1024, 2 * 1024, 60 * 60000, 0,
        "MsdfStrokeUbo".to_string(),
    );
    res_mgr.register::<R12>(
        1 * 1024, 2 * 1024, 60 * 60000, 0,
        "CanvasTextStrokeColorUbo".to_string(),
    );
    res_mgr
}

#[test]
pub fn test() {
    let total_capacity: usize = 67108864;
    let mut res_mgr = create_res_mgr(total_capacity);
    let texture = res_mgr.fetch_map::<R1>(0).unwrap();
    let buffer = res_mgr.fetch_map::<R3>(0).unwrap();
    let blend_state = res_mgr.fetch_map::<R6>(0).unwrap();
    let sampler_res = res_mgr.fetch_map::<R4>(0).unwrap();
    let u_color_ubo = res_mgr.fetch_map::<R9>(0).unwrap();
    let msdf_stroke_ubo = res_mgr.fetch_map::<R11>(0).unwrap();
    let canvas_text_stroke_color_ubo = res_mgr.fetch_map::<R12>(0).unwrap();
    let geometry_res = res_mgr.fetch_map::<R2>(0).unwrap();

    let texture = unsafe { &mut *(&*texture as *const ResMap<R1> as *mut ResMap<R1>) };
    let buffer = unsafe { &mut *(&*buffer as *const ResMap<R3> as *mut ResMap<R3>) };
    let blend_state = unsafe { &mut *(&*blend_state as *const ResMap<R6> as *mut ResMap<R6>) };
    let sampler_res = unsafe { &mut *(&*sampler_res as *const ResMap<R4> as *mut ResMap<R4>) };
    let u_color_ubo = unsafe { &mut *(&*u_color_ubo as *const ResMap<R9> as *mut ResMap<R9>) };
    let msdf_stroke_ubo =
        unsafe { &mut *(&*msdf_stroke_ubo as *const ResMap<R11> as *mut ResMap<R11>) };
    let canvas_text_stroke_color_ubo =
        unsafe { &mut *(&*canvas_text_stroke_color_ubo as *const ResMap<R12> as *mut ResMap<R12>) };
    let geometry_res = unsafe { &mut *(&*geometry_res as *const ResMap<R2> as *mut ResMap<R2>) };

    texture.create(Atom::from("__$text"), R1 {}, 262144, 0);
    buffer.create(3902250154, R3 {}, 32, 0);
    buffer.create(1519695964, R3 {}, 12, 0);
    blend_state.create(2905594028, R6 {}, 0, 0);
    blend_state.create(3006512311, R6 {}, 0, 0);
    sampler_res.create(308248423, R4 {}, 0, 0);
    sampler_res.create(2591543091, R4 {}, 0, 0);
    u_color_ubo.create(2106312588, R9 {}, 0, 0);
    msdf_stroke_ubo.create(3879787636, R11 {}, 0, 0);
    canvas_text_stroke_color_ubo.create(1145791972, R12 {}, 0, 0);
    u_color_ubo.create(796366362, R9 {}, 0, 0);

    res_mgr.collect(0);

    u_color_ubo.create(2492188942, R9 {}, 0, 0);
    u_color_ubo.create(4246038113, R9 {}, 0, 0);
    u_color_ubo.create(2564464371, R9 {}, 0, 0);
    u_color_ubo.create(1601737751, R9 {}, 0, 0);

    texture.create(Atom::from("1"), R1 {}, 131072, 0);
    texture.create(Atom::from("2"), R1 {}, 65536, 0);
    texture.create(Atom::from("3"), R1 {}, 1572864, 0);

    buffer.create(1156469915, R3 {}, 32, 0);
    geometry_res.create(940515885, R2 {}, 0, 0);
    buffer.create(2294013149, R3 {}, 32, 0);
    geometry_res.create(1197965350, R2 {}, 0, 0);
    buffer.create(2418548769, R3 {}, 32, 0);
    geometry_res.create(203047359, R2 {}, 0, 0);

    texture.create(Atom::from("4"), R1 {}, 4194304, 0);

    buffer.create(2402797892, R3 {}, 32, 0);
    geometry_res.create(1733445847, R2 {}, 0, 0);
    texture.create(Atom::from("4"), R1 {}, 4194304, 0);
    texture.create(Atom::from("5"), R1 {}, 4194304, 0);

    buffer.create(1, R3 {}, 32, 0);
    geometry_res.create(1, R2 {}, 0, 0);
    buffer.create(2, R3 {}, 32, 0);
    geometry_res.create(2, R2 {}, 0, 0);
    res_mgr.collect(2000);

    texture.create(Atom::from("6"), R1 {}, 1048576, 0);

    buffer.create(3, R3 {}, 32, 0);
    geometry_res.create(3, R2 {}, 0, 0);
    buffer.create(4, R3 {}, 32, 0);
    geometry_res.create(4, R2 {}, 0, 0);
    u_color_ubo.create(1009613414, R9 {}, 0, 0);
    buffer.create(5, R3 {}, 32, 0);
    geometry_res.create(5, R2 {}, 0, 0);
    buffer.create(6, R3 {}, 32, 0);
    geometry_res.create(6, R2 {}, 0, 0);
    buffer.create(7, R3 {}, 32, 0);
    geometry_res.create(7, R2 {}, 0, 0);
    buffer.create(8, R3 {}, 32, 0);
    geometry_res.create(8, R2 {}, 0, 0);
    buffer.create(9, R3 {}, 32, 0);
    geometry_res.create(9, R2 {}, 0, 0);
    u_color_ubo.create(4173812235, R9 {}, 0, 0);
    texture.create(Atom::from("7"), R1 {}, 1572864, 0);
    texture.create(Atom::from("8"), R1 {}, 65536, 0);
    buffer.create(10, R3 {}, 32, 0);
    geometry_res.create(10, R2 {}, 0, 0);
    texture.create(Atom::from("16"), R1 {}, 1048576, 0);
    texture.create(Atom::from("9"), R1 {}, 2097152, 0);
    texture.create(Atom::from("10"), R1 {}, 1048576, 0);
    texture.create(Atom::from("11"), R1 {}, 2097152, 0);
    texture.create(Atom::from("12"), R1 {}, 2097152, 0);
    buffer.create(11, R3 {}, 32, 0);
    geometry_res.create(11, R2 {}, 0, 0);
    buffer.create(12, R3 {}, 32, 0);
    geometry_res.create(12, R2 {}, 0, 0);
    buffer.create(13, R3 {}, 32, 0);
    geometry_res.create(13, R2 {}, 0, 0);
    buffer.create(14, R3 {}, 32, 0);
    geometry_res.create(14, R2 {}, 0, 0);

    texture.create(Atom::from("13"), R1 {}, 2097152, 0);

    res_mgr.collect(3000);
    println!(
        "xxxxxxxxxxxxxxxxxxxxxxxxxx4: {:?}",
        texture.cache.get_max_capacity()
    );

    // let r1 = res_mgr.fetch_map().unwrap();
}
