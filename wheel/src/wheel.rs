/// Thread unsafe wheel structure, which supports quick deletion by index, with a precision of 10 milliseconds.
/// 
/// 
///
/// 
use std::cmp::{Ord, Ordering};
use std::mem::{replace, swap};
use std::fmt::{Debug, Formatter, Result as FResult};

use heap::heap::Heap;
use dyn_uint::{UintFactory, ClassFactory};

static START:[u8; 4] = [0, 100, 160, 220];
static CAPACITY:[u8; 4] = [100, 60, 60, 24];
static UNIT:[u32; 4] = [10, 1000, 60000, 3600000];

pub struct Wheel<T>{
	arr: [Vec<(Item<T>, usize)>; 244],//毫秒精度为10， 秒，分钟， 小时精度为1
    zero_arr:Vec<(Item<T>, usize)>,
    zero_cache: Vec<(Item<T>, usize)>,
	heap: Heap<Item<T>>,
	point:[u8; 4],
	time: u64,//当前时间
}

impl<T> Wheel<T>{

	//Create a wheel to support four rounds.
	pub fn new() -> Self{
		let arr: [Vec<(Item<T>, usize)>; 244] = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new()];
		Wheel{
			arr: arr,
			zero_arr: Vec::new(),
			zero_cache: Vec::new(),
			heap: Heap::new(Ordering::Less),
			point:[0, 0, 0, 0],
			time:0
		}
	}

	//Setting wheel time
    #[inline]
	pub fn set_time(&mut self, ms: u64){
		self.time = ms;
	}

    #[inline]
    pub fn get_time(&mut self) -> u64{
		self.time
	}

    #[inline]
    pub fn zero_size(&self) -> usize{
		self.zero_arr.len()
	}

    #[inline]
	pub fn get_zero(&mut self) -> Vec<(Item<T>, usize)>{
		replace(&mut self.zero_arr, replace(&mut self.zero_cache, Vec::new()))
	}

    #[inline]
    pub fn set_zero_cache(&mut self, v: Vec<(Item<T>, usize)>){
		replace(&mut self.zero_cache, v);
	}

	//插入元素
	pub fn insert< F: UintFactory + ClassFactory<usize>>(&mut self, item: Item<T>, index: usize, index_factory: &mut F){
		// 计算时间差
		let mut diff = sub(item.time_point, self.time);

		//如果时间差为0， 则将其插入到zero_arr（特殊处理0毫秒）
		if diff == 0 {
            index_factory.store(index, self.zero_arr.len());
            index_factory.set_class(index, 244);
			self.zero_arr.push((item, index));
			return;
		}
		if diff >= 90061000{
            index_factory.set_class(index, 245);
			return self.heap.push(item, index, index_factory);
		}
		diff = diff - 1;
		
		if diff < 1000{
			self.insert_ms((item, index), diff, index_factory);
		}else if diff < 61000{
			self.insert_wheel((item, index), 1, diff, index_factory);
		}else if diff < 3661000{
			self.insert_wheel((item, index), 2, diff, index_factory);
		}else{
			self.insert_wheel((item, index), 3, diff, index_factory);
		}
	}

	pub fn roll< F: UintFactory + ClassFactory<usize>>(&mut self, index_factory: &mut F) -> Vec<(Item<T>, usize)>{
		self.time += 10;
		self.forward(0, index_factory)
	}

	// pub fn try_remove< F: UintFactory + ClassFactory<usize>>(&mut self, index: usize, index_factory: &mut F) -> Option<(Item<T>, usize)>{
	// 	let i = index_factory.load(index);
	// 	if (i >> 2) == 0 {
	// 		return None;
	// 	}
	// 	let t = i & 3; //类型
	// 	if t == 0{
	// 		self.heap.try_remove(index, index_factory)
	// 	}else if t == 1{
	// 		let index = resolve_index(i);
	// 		let arr = &mut self.arr[index.0];
	// 		if index.1 >= arr.len(){
	// 			return None;
	// 		}
	// 		Some(Wheel::delete(arr, index.1, i, index_factory))
	// 	}else{
	// 		None
	// 	}
	// }

	//Panics if index is out of bounds.
	pub fn delete< F: UintFactory + ClassFactory<usize>>(&mut self, class: usize, index:usize, index_factory: &mut F) -> Option<(Item<T>, usize)> {
		if class == 245 { //heap的类型为245
			unsafe { Some(self.heap.delete(index, index_factory)) }
		} else if class == 244 {
            Wheel::delete_wheel(&mut self.zero_arr, index, index_factory)
        } else {//wheel的类型为1
			Wheel::delete_wheel(&mut self.arr[class], index, index_factory)
		}
	}

	//clear all elem
	pub fn clear(&mut self){
		self.heap.clear();
		for v in self.arr.iter_mut() {
			v.clear();
		}
		self.zero_arr.clear();
		self.zero_cache.clear();
		self.point = [0,0,0,0];
		self.time = 0;
	}

	//插入到毫秒轮
	fn insert_ms< F: UintFactory + ClassFactory<usize>>(&mut self, item: (Item<T>, usize), diff: u64, index_factory: &mut F){
		let i = (next_tail(self.point[0], (diff/10) as u8, 100)) as usize;
        index_factory.store(item.1, self.arr[i].len());
        index_factory.set_class(item.1, i);
		self.arr[i].push(item);
	}

	//秒，分钟，小时轮的插入方法
	fn insert_wheel< F: UintFactory + ClassFactory<usize>>(&mut self, item: (Item<T>, usize), layer: usize, diff: u64, index_factory: &mut F){
		let i = (next_tail(self.point[layer], (diff/(UNIT[layer] as u64)) as u8 - 1, CAPACITY[layer]) + START[layer]) as usize;
		index_factory.store(item.1, self.arr[i].len());
        index_factory.set_class(item.1, i);
		self.arr[i].push(item);
	}

	fn delete_wheel< F: UintFactory + ClassFactory<usize>>(arr: &mut Vec<(Item<T>, usize)>, index: usize, index_factory: &mut F) -> Option<(Item<T>, usize)> {
		if let Some(mut r) = arr.pop() {
			if index < arr.len(){
				index_factory.store(r.1, index);
				swap(&mut r, &mut arr[index]);
			}
			return Some(r);
		}

		None
	}

	//前进一个单位
	fn forward< F: UintFactory + ClassFactory<usize>>(&mut self, layer: usize, index_factory: &mut F) -> Vec<(Item<T>, usize)>{
		let point = self.point[layer] as usize;
		let s = START[layer] as usize;
		let r = replace(&mut self.arr[point + s], Vec::new());
		self.point[layer] = next_tail(point as u8, 1, CAPACITY[layer]);
		if self.point[layer] == 0{
			let above = match layer > 2{
				true => self.get_from_heap(index_factory),
				false => self.forward(layer + 1, index_factory)
			};
			for v in above.into_iter() {
				let diff = match v.0.time_point > self.time{
					true => v.0.time_point - self.time - 1,
					false => 0,
				};
				match diff{
					0..1000 => self.insert_ms(v, diff, index_factory),
					1000..61000 => self.insert_wheel(v, 1, diff, index_factory),
					61000..3661000 => self.insert_wheel(v, 2, diff, index_factory),
					_ => self.insert_wheel(v, 3, diff, index_factory)
				}
			}
		}
		r
	}

	fn get_from_heap< F: UintFactory + ClassFactory<usize>>(&mut self, index_factory: &mut F) -> Vec<(Item<T>, usize)>{
		let mut r = Vec::new();
		loop {
			match self.heap.len() > 0{
				true => {
                    let v = unsafe {self.heap.get_unchecked(0)};
					if sub(v.time_point, self.time) >= 90061000 {
						break;
					}
				}
				false => break
			};
			r.push(unsafe {self.heap.delete(0, index_factory) });
		}
		r
	}
}

impl<T: Debug> Debug for Wheel<T> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        let mut arr_str = "[".to_string();
        let mut i = 1;
        for v in self.arr.iter(){
            arr_str += "(";
            arr_str += i.to_string().as_str();
            arr_str += ",";
            arr_str += &format!("{:?}", v);
            arr_str += ")";
            arr_str += ",";
            i += 1;
        }
        arr_str += "]";

        write!(fmt,
r##"Wheel( 
    arr: {:?},
    zero_arr: {:?},
    zero_cache:{:?},
    heap:{:?},
    point:{:?},
    time:{}
)"##,
               arr_str,
               self.zero_arr,
               self.zero_cache,
               self.heap,
               self.point,
               self.time,
        )
    }
}

#[inline]
fn next_tail(cur: u8, span: u8, c: u8) -> u8{
	(cur + span)%c
}

#[inline]
fn sub(x: u64, y: u64) -> u64{
	match x > y{
		true => x - y,
		false => 0
	}
}

#[derive(Clone, Debug)]
pub struct Item<T> {
	pub elem: T,
	pub time_point: u64,
}

impl<T> Item<T>{
    pub fn new(elem: T, time_point: u64) -> Item<T>{
        Item{
            elem,
            time_point
        }
    }
}

impl<T> Ord for Item<T> {
    fn cmp(&self, other: &Item<T>) -> Ordering {
        self.time_point.cmp(&other.time_point)
    }
}

impl<T> PartialOrd for Item<T> {
    fn partial_cmp(&self, other: &Item<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for Item<T> {
    fn eq(&self, other: &Item<T>) -> bool {
        self.time_point == other.time_point
    }
}

impl<T> Eq for Item<T> {
}
