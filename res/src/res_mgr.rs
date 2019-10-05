
// 资源管理器， 管理多个资源表。有总内存上限的控制。
// 初始化时，设置的每个资源表内的LRU的max_capacity和min_capacity的差，就是每个LRU的权重。
// 如果有LRU有空闲， 则会减少其max_capacity, 按权重提高那些满的LRU的max_capacity


use std::any::{ TypeId };

use hash::XHashMap;
use share::{Share};

use super::res_map::{Res, ResMap, ResCollect, StateInfo};

pub static CAPACITY: usize = 16*1024*1024;

pub struct ResMgr {
    tables: XHashMap<TypeId, (Share<dyn ResCollect>, [usize;3])>,
    total_capacity: usize,
    weight: usize,
    min_capacity: usize,
}
impl Default for ResMgr {
	fn default() -> Self {
		ResMgr::with_capacity(CAPACITY)
	}
}
impl ResMgr {
    pub fn with_capacity(total_capacity: usize) -> Self{
        ResMgr{
            tables: XHashMap::default(),
            total_capacity,
            weight: 0,
            min_capacity: 0,
        }
    }

	pub fn mem_size(&self) -> usize {
		let mut r = 0;
		for (_, v) in self.tables.iter() {
			r += v.0.mem_size();
		}
		r
	}

    /// 注册指定类型的资源表。 参数为资源表的3种lru的配置。 [min_capacity1, max_capacity1, timeout1, min_capacity2, max_capacity2, timeout2, min_capacity3, max_capacity3, timeout3]。 如果不使用后2种，直接将min_capacity, max_capacity都设成0。
    #[inline]
    pub fn register<T: Res + 'static>(&mut self, configs: [usize; 9]) {
        let arr = [configs[1]-configs[0], configs[4]-configs[3], configs[7]-configs[6]]; // 权重数组
		let total: usize = arr.iter().sum();
		self.weight += total;
		self.min_capacity += configs[0] + configs[3] + configs[6];
		let weight = &mut self.weight;
		let min_capacity = &mut self.min_capacity;
		self.tables.entry(TypeId::of::<T>()).and_modify(|e| {
			let r = match e.0.clone().downcast::<ResMap<T>>() {
				Ok(r) => r,
				Err(_) => return,
			};
			let old = get_mut(&*(r)).modify_config(&configs);
			let old_arr = e.1;
			let old_total: usize = old_arr.iter().sum();
			e.1 = arr;
			*weight -= old_total;
			*min_capacity -= old[0].0 + old[1].0 + old[2].0;
		}).or_insert((Share::new(ResMap::<T>::with_config(&configs)), arr));
    }

	pub fn fetch_map<T: Res>(&self) -> Option<Share<ResMap<T>>>{
		match self.tables.get(&TypeId::of::<T>()) {
			Some(i) => match i.0.clone().downcast::<ResMap<T>>() {
				Ok(r) => Some(r),
				Err(_) => None,
			},
			_ => None,
		}
	}

	pub fn get<T: Res + 'static>(&self, name: &<T as Res>::Key) -> Option<Share<T>>{
		match self.tables.get(&TypeId::of::<T>()) {
			Some(i) => match i.0.clone().downcast::<ResMap<T>>() {
				Ok(r) => match get_mut(&*r).get(name) {
					Some(r) => Some(r),
					None => None,
				},
				Err(_) => None
			},
			_ => None,
		}
	}

	#[inline]
	pub fn create<T: Res + 'static>(&mut self, name: T::Key, value: T, cost: usize, rtype: usize) -> Share<T> {
		match self.tables.get(&TypeId::of::<T>()) {
			Some(i) => match i.0.clone().downcast::<ResMap<T>>() {
				Ok(r) => get_mut(&*r).create(name, value, cost, rtype),
				Err(_) => panic!("downcast error!"),
			},
			None => panic!("TypeId not found!"),
		}
	}

	#[inline]
	pub fn remove<T: Res + 'static>(&mut self, name: &<T as Res>::Key) -> Option<Share<T>> {
		match self.tables.get(&TypeId::of::<T>()) {
			Some(i) => match i.0.clone().downcast::<ResMap<T>>() {
				Ok(r) => get_mut(&*r).remove(name),
				Err(_) => None
			},
			_ => None,
		}
	}
	// 整理方法， 将无人使用的资源放入到LruCache， 清理过时的资源
	// 就是LruMgr有总内存上限， 按权重分给其下的LRU。 如果有LRU有空闲， 则会减少其max_size, 按权重提高那些满的LRU的max_size
	pub fn collect(&mut self, now: usize) {
		let capacity = self.total_capacity as isize - self.min_capacity as isize;
		let capacity = if capacity < 0 {
			0
		} else {
			capacity as usize
		};
		let mut up_size = 0; // 超过权重的总大小
		let mut down_size = 0; // 小于权重的总大小
		let mut vec = Vec::new(); // map引用
		let mut up_full = Vec::new(); // 超过权重并满了的map_index
		let mut up_ok = Vec::new(); // 超过权重并Ok的map_index

		for v in self.tables.values() {
			let vm = &*(v.0);
			let map = unsafe{&mut *(vm as *const dyn ResCollect as *mut dyn ResCollect)};
			let arr = map.collect(now);
			let mut i = 0;
			for ss in arr.iter() {
				let calc_max = capacity / self.weight * v.1[i]; // 该lru根据权重算出来的可增加的内存总量，如果加上min_capacity则是最大容量max_capacity
				match ss {
					&StateInfo::Full(min, size) => {
						// 如果当前大小小于权重大小，则扩大容量到权重大小
						if size < min + calc_max {
							map.set_max_capacity(i, min + calc_max);
						}else if size > min + calc_max {
							up_size += size - (min + calc_max);
							up_full.push((vec.len(), i, size));
						}
					},
					&StateInfo::Ok(min, size) => {
						if size < min + calc_max {
							down_size += min + calc_max - size;
						}else if size > min + calc_max {
							up_size += size - (min + calc_max);
							up_ok.push((vec.len(), i, size));
						}
					},
					&StateInfo::Free(min, _size, right_size) => {
						map.set_max_capacity(i, right_size);
						if right_size < min + calc_max {
							down_size += min + calc_max - right_size;
						}else{
							up_size += right_size - (min + calc_max);
							up_ok.push((vec.len(), i, right_size));
						}
					},
					_ => ()
				}
				i += 1;
			}
			vec.push(map);
		}
		if up_size > down_size { // 如果超过的权重比小于的权重大，表示需要控制大小，将up_full和up_ok的lru的容量变小，
			let del = (up_size - down_size) / (up_full.len() + up_ok.len());
			for v in up_full {
				let map = unsafe {vec.get_unchecked_mut(v.0)};
				map.set_max_capacity(v.1, if v.2 > del {v.2 - del}else{0});
			}
			for v in up_ok {
				let map = unsafe {vec.get_unchecked_mut(v.0)};
				map.set_max_capacity(v.1, if v.2 > del {v.2 - del}else{0});
			}
		}else if up_size < down_size { // 表示有空闲大小， 将up_full的lru的容量扩大
			let add = (down_size - up_size) / up_full.len();
			for v in up_full {
				let map = unsafe {vec.get_unchecked_mut(v.0)};
				map.set_max_capacity(v.1, v.2 + add);
			}
		}
	}
}

fn get_mut<T:Res>(map: &ResMap<T>) -> &mut ResMap<T> {
	unsafe{&mut *(map as *const ResMap<T> as *mut ResMap<T>)}
}