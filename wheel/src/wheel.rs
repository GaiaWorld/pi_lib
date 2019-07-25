/// Thread unsafe wheel structure, which supports quick deletion by index, with a precision of 10 milliseconds.
/// 
/// 
///
/// 
use std::cmp::{Ord, Ordering};
use std::mem::{replace, swap};
use std::fmt::{Debug, Formatter, Result as FResult};

use heap::heap::Heap;
use index_class::{IndexClassFactory};
use ver_index::VerIndex;

static START:[u8; 4] = [0, 100, 160, 220];
static CAPACITY:[u8; 4] = [100, 60, 60, 24];
static UNIT:[u32; 4] = [10, 1000, 60000, 3600000];

pub struct Wheel<T, I: VerIndex>{
	arr: [Vec<(Item<T>, I::ID)>; 244],//毫秒精度为10， 秒，分钟， 小时精度为1
    zero_arr:Vec<(Item<T>, I::ID)>,
    zero_cache: Vec<(Item<T>, I::ID)>,
	heap: Heap<Item<T>, I::ID>,
	point:[u8; 4],
	time: u64,//当前时间
}

impl<T, I: VerIndex> Wheel<T, I>{

	//Create a wheel to support four rounds.
	pub fn new() -> Self{
		let arr: [Vec<(Item<T>, I::ID)>; 244] = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new()];
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
	pub fn get_zero(&mut self, vec: Vec<(Item<T>, I::ID)>) -> Vec<(Item<T>, I::ID)>{
		replace(&mut self.zero_arr, vec)
	}
    #[inline]
    pub fn replace_zero_cache(&mut self, vec: Vec<(Item<T>, I::ID)>) -> Vec<(Item<T>, I::ID)>{
		replace(&mut self.zero_cache, vec)
	}

	//插入元素
	pub fn insert(&mut self, item: Item<T>, id: I::ID, factory: &mut IndexClassFactory<usize, (), I>){
		// 计算时间差
		let mut diff = sub(item.time_point, self.time);

		//如果时间差为0， 则将其插入到zero_arr（特殊处理0毫秒）
		if diff == 0 {
			let v = unsafe {factory.get_unchecked_mut(id) };
			v.index = self.zero_arr.len();
			v.class = 244;
            // factory.store(index, self.zero_arr.len());
            // factory.set_class(index, 244);
			self.zero_arr.push((item, id));
			return;
		}
		if diff >= 90061000{
			unsafe {factory.get_unchecked_mut(id) }.class=245;
            //factory.set_class(index, 245);
			return self.heap.push(item, id, factory);
		}
		diff = diff - 1;
		
		if diff < 1000{
			self.insert_ms((item, id), diff, factory);
		}else if diff < 61000{
			self.insert_wheel((item, id), 1, diff, factory);
		}else if diff < 3661000{
			self.insert_wheel((item, id), 2, diff, factory);
		}else{
			self.insert_wheel((item, id), 3, diff, factory);
		}
	}

	pub fn roll(&mut self, factory: &mut IndexClassFactory<usize, (), I>) -> Vec<(Item<T>, I::ID)>{
		self.time += 10;
		self.forward(0, factory)
	}

	// pub fn try_remove(&mut self, index: usize, factory: &mut ClassFactory<usize, ()>) -> Option<(Item<T>, usize)>{
	// 	let i = factory.load(index);
	// 	if (i >> 2) == 0 {
	// 		return None;
	// 	}
	// 	let t = i & 3; //类型
	// 	if t == 0{
	// 		self.heap.try_remove(index, factory)
	// 	}else if t == 1{
	// 		let index = resolve_index(i);
	// 		let arr = &mut self.arr[index.0];
	// 		if index.1 >= arr.len(){
	// 			return None;
	// 		}
	// 		Some(Wheel::delete(arr, index.1, i, factory))
	// 	}else{
	// 		None
	// 	}
	// }

	//Panics if index is out of bounds.
	pub fn delete(&mut self, class: usize, index:usize, factory: &mut IndexClassFactory<usize, (), I>) -> (Item<T>, I::ID){
		if class == 245 { //heap的类型为245
			unsafe { self.heap.delete(index, factory) }
		} else if class == 244 {
            Wheel::delete_wheel(&mut self.zero_arr, index, factory)
        } else {//wheel的类型为1
			Wheel::delete_wheel(&mut self.arr[class], index, factory)
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
	fn insert_ms(&mut self, item: (Item<T>, I::ID), diff: u64, factory: &mut IndexClassFactory<usize, (), I>){
		let i = (next_tail(self.point[0], (diff/10) as u8, 100)) as usize;
		let v = unsafe {factory.get_unchecked_mut(item.1) };
		v.index = self.arr[i].len();
		v.class = i;
		self.arr[i].push(item);
	}

	//秒，分钟，小时轮的插入方法
	fn insert_wheel(&mut self, item: (Item<T>, I::ID), layer: usize, diff: u64, factory: &mut IndexClassFactory<usize, (), I>){
		let i = (next_tail(self.point[layer], (diff/(UNIT[layer] as u64)) as u8 - 1, CAPACITY[layer]) + START[layer]) as usize;
		let v = unsafe {factory.get_unchecked_mut(item.1) };
		v.index = self.arr[i].len();
		v.class = i;
		self.arr[i].push(item);
	}

	fn delete_wheel(arr: &mut Vec<(Item<T>, I::ID)>, index: usize, factory: &mut IndexClassFactory<usize, (), I>) -> (Item<T>, I::ID){
		let mut r = arr.pop().unwrap();
		if index < arr.len(){
			unsafe {factory.get_unchecked_mut(r.1) }.index = index;
			swap(&mut r, &mut arr[index]);
		}
		r
	}

	//前进一个单位
	fn forward(&mut self, layer: usize, factory: &mut IndexClassFactory<usize, (), I>) -> Vec<(Item<T>, I::ID)>{
		let point = self.point[layer] as usize;
		let s = START[layer] as usize;
		let r = replace(&mut self.arr[point + s], Vec::new());
		self.point[layer] = next_tail(point as u8, 1, CAPACITY[layer]);
		if self.point[layer] == 0{
			let above = match layer > 2{
				true => self.get_from_heap(factory),
				false => self.forward(layer + 1, factory)
			};
			for v in above.into_iter() {
				let diff = match v.0.time_point > self.time{
					true => v.0.time_point - self.time - 1,
					false => 0,
				};
				match diff{
					0..1000 => self.insert_ms(v, diff, factory),
					1000..61000 => self.insert_wheel(v, 1, diff, factory),
					61000..3661000 => self.insert_wheel(v, 2, diff, factory),
					_ => self.insert_wheel(v, 3, diff, factory)
				}
			}
		}
		r
	}

	fn get_from_heap(&mut self, factory: &mut IndexClassFactory<usize, (), I>) -> Vec<(Item<T>, I::ID)>{
		// TODO 不需要用vec做中间缓存， 应该直接放入到对应的槽位
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
			r.push(unsafe {self.heap.delete(0, factory) });
		}
		r
	}
}

impl<T: Debug, I:VerIndex> Debug for Wheel<T, I> where T: Debug {
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
