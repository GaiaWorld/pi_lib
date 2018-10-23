/// Thread unsafe wheel structure, which supports quick deletion by index, with a precision of 10 milliseconds.
/// 
/// 

use std::sync::atomic::{AtomicUsize, Ordering as AOrd};
use std::sync::Arc;
use std::cmp::{Ord, Ordering};
use std::mem::{replace, swap};

use heap::Heap;

static START:[u8; 4] = [0, 100, 160, 220];
static CAPACITY:[u8; 4] = [100, 60, 60, 24];
static UNIT:[u32; 4] = [10, 1000, 60000, 3600000];

pub struct Wheel<T>{
	arr: [Vec<(Item<T>, Arc<AtomicUsize>)>; 244],//毫秒精度为10， 秒，分钟， 小时精度为1
    zero_arr:Vec<(Item<T>, Arc<AtomicUsize>)>,
    zero_cache: Vec<(Item<T>, Arc<AtomicUsize>)>,
	heap: Heap<Item<T>>,
	point:[u8; 4],
	pub time: u64,//当前时间
}

impl<T> Wheel<T>{

	//Create a wheel to support four rounds.
	pub fn new() -> Self{
		let arr: [Vec<(Item<T>, Arc<AtomicUsize>)>; 244] = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new()];
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
	pub fn set_time(&mut self, ms: u64){
		self.time = ms;
	}

	//插入元素
	pub fn insert(&mut self, item: Item<T>) -> Arc<AtomicUsize>{
		// 计算时间差
		let mut diff = sub(item.time_point, self.time);

		//如果时间差为0， 则将其插入到zero_arr（特殊处理0毫秒）
		if diff == 0 {
			let index = Arc::new(new_index(244,  self.zero_arr.len()));
			self.zero_arr.push((item, index.clone()));
			return index;
		}
		if diff >= 90061000{
			return self.heap.push(item);
		}
		let index = Arc::new(AtomicUsize::new(0));
		diff = diff - 1;
		
		if diff < 1000{
			self.insert_ms((item, index.clone()), diff);
		}else if diff < 61000{
			self.insert_wheel((item, index.clone()), 1, diff);
		}else if diff < 3661000{
			self.insert_wheel((item, index.clone()), 2, diff);
		}else{
			self.insert_wheel((item, index.clone()), 3, diff);
		}
		index
	}

	pub fn zero_size(&self) -> usize{
		self.zero_arr.len()
	}

	pub fn get_zero(&mut self) -> Vec<(Item<T>, Arc<AtomicUsize>)>{
		replace(&mut self.zero_arr, replace(&mut self.zero_cache, Vec::new()))
	}

    pub fn set_zero_cache(&mut self, v: Vec<(Item<T>, Arc<AtomicUsize>)>){
		replace(&mut self.zero_cache, v);
	}

	pub fn roll(&mut self) -> Vec<(Item<T>, Arc<AtomicUsize>)>{
		self.time += 10;
		self.forward(0)
	}

	pub fn try_remove(&mut self, index: &Arc<AtomicUsize>) -> Option<Item<T>>{
		let i = index.load(AOrd::Relaxed);
		if (i >> 2) == 0 {
			return None;
		}
		let t = i & 3; //类型
		if t == 0{
			self.heap.try_remove(index)
		}else if t == 1{
			let index = load_index(i);
			let arr = &mut self.arr[index.0];
			if index.1 >= arr.len(){
				return None;
			}
			Some(Wheel::delete(arr, index.1, i))
		}else{
			None
		}
	}

	//Panics if index is out of bounds.
	pub fn remove(&mut self, index: &Arc<AtomicUsize>) -> Item<T>{
		let i = index.load(AOrd::Relaxed);
		let t = i & 3;
		if t == 0 { //heap的类型为0
			self.heap.remove(index)
		}else if t == 1 {//wheel的类型为1
			let index = load_index(i);
			Wheel::delete(&mut self.arr[index.0], index.1, i)
		}else{
			panic!("type is err!, can't remove from wheel");
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
	fn insert_ms(&mut self, item: (Item<T>, Arc<AtomicUsize>), diff: u64){
		let i = (next_tail(self.point[0], (diff/10) as u8, 100)) as usize;
		store_index(i, self.arr[i].len(), &item.1);
		self.arr[i].push(item);
	}

	//秒，分钟，小时轮的插入方法
	fn insert_wheel(&mut self, item: (Item<T>, Arc<AtomicUsize>), layer: usize, diff: u64){
		let i = (next_tail(self.point[layer], (diff/(UNIT[layer] as u64)) as u8 - 1, CAPACITY[layer]) + START[layer]) as usize;
		store_index(i, self.arr[i].len(), &item.1);
		self.arr[i].push(item);
	}

	fn delete(arr: &mut Vec<(Item<T>, Arc<AtomicUsize>)>, index: usize, i: usize) -> Item<T>{
		let mut r = arr.pop().unwrap();
		if index < arr.len(){
			r.1.store(i, AOrd::Relaxed);
			swap(&mut r, &mut arr[index]);
		}
		r.1.store(0, AOrd::Relaxed);
		r.0
	}

	//前进一个单位
	fn forward(&mut self, layer: usize) -> Vec<(Item<T>, Arc<AtomicUsize>)>{
		if layer > 2{
			println!("layer-------------{}", layer);
		}
		let point = self.point[layer] as usize;
		let s = START[layer] as usize;
		let r = replace(&mut self.arr[point + s], Vec::new());
		self.point[layer] = next_tail(point as u8, 1, CAPACITY[layer]);
		if self.point[layer] == 0{
			let above = match layer > 2{
				true => self.get_from_heap(),
				false => self.forward(layer + 1)
			};
			for v in above.into_iter() {
				let diff = match v.0.time_point > self.time{
					true => v.0.time_point - self.time - 1,
					false => 0,
				};
				match diff{
					0..1000 => self.insert_ms(v, diff),
					1000..61000 => self.insert_wheel(v, 1, diff),
					61000..3661000 => self.insert_wheel(v, 2, diff),
					_ => self.insert_wheel(v, 3, diff)
				}
			}
		}
		r
	}

	fn get_from_heap(&mut self) -> Vec<(Item<T>, Arc<AtomicUsize>)>{
		let mut r = Vec::new();
		let mut flag = true;
		while flag {
			match self.heap.get_top(){
				Some(v) => {
					if sub(v.time_point, self.time) < 90061000{
						r.push(self.heap.remove_top());
					}else{
						flag = false;
					}
				}
				None => {
					flag = false;
				}
			};
		}
		r
	}
}

//new index in AtomicUsize, The last two bytes represent the type, and 1 means the wheel.
fn new_index(index1: usize, index2: usize) -> AtomicUsize{
	AtomicUsize::new(((index2*245 + index1 + 1) << 2) + 1)
}

//store index in AtomicUsize, The last two bytes represent the type, and 1 means the wheel. 
#[inline]
fn store_index(index1: usize, index2: usize, dst: &Arc<AtomicUsize>){
	dst.store(((index2*245 + index1 + 1) << 2) + 1, AOrd::Relaxed);
}

//load index from AtomicUsize, The last two bytes represent the type, and 1 means the wheel.
#[inline]
fn load_index(index: usize) -> (usize, usize){
	let index = (index >> 2) - 1 ;
	(index%245, index/245)
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

#[derive(Clone)]
pub struct Item<T> {
	pub elem: T,
	pub time_point: u64,
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

#[test]
fn test(){
	let mut wheel = Wheel::new();
	let mut arr = Vec::new();
	let times = [0, 10, 1000, 3000, 3100, 50, 60000, 61000, 3600000, 3500000, 86400000, 86600000];
	let expect_index = [(244, 0), (0, 0), (99, 0), (101, 0), (102, 0), (4, 0), (158, 0), (159, 0), (218, 0), (217, 0), (242, 0), (243, 0)];
	let mut i = 0;
	//测试插入到轮中的元素位置是否正确
	for v in times.iter(){
		let index = wheel.insert(Item{elem: v.clone(), time_point: v.clone() as u64});
		assert_eq!(&load_index(index.load(AOrd::Relaxed)), &expect_index[i]);
		arr.push(index);
		i += 1;
	}

	//测试插入到堆中的元素位置是否正确
	let heap_elem = 90061001;
	let index = wheel.insert(Item{elem: heap_elem, time_point: heap_elem as u64});
	assert_eq!(index.load(AOrd::Relaxed), 4);

	//滚动一次， 只有时间为10毫秒的元素被取出
	let r = wheel.roll();
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 10);

	//滚动三次， 不能取出任何元素
	for _i in 1..4{
		let r = wheel.roll();
		assert_eq!(r.len(), 0);
	}

	//滚动1次， 只有时间为50毫秒的元素被取出（滚动第五次）
	let r = wheel.roll();
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 50);

	//滚动94次， 不能取出任何元素（滚动到第99次）
	for _i in 1..95{
		let r = wheel.roll();
		assert_eq!(r.len(), 0);
	}

	//滚动1次， 只有时间为1000毫秒的元素被取出（滚动到第100次）
	let r = wheel.roll();
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 1000);

	//滚动199次， 不能取出任何元素（滚动到第299次）
	for _i in 1..200{
		let r = wheel.roll();
		assert_eq!(r.len(), 0);
	}

	//滚动1次， 只有时间为3000毫秒的元素被取出（滚动到第300次）
	let r = wheel.roll();
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 3000);

	let r = wheel.remove(&Arc::new(new_index(expect_index[7].0, expect_index[7].1)));
	assert_eq!(r.time_point, 61000);

	
	let r = wheel.remove(&Arc::new(new_index(expect_index[6].0, expect_index[6].1)));
	assert_eq!(r.time_point, 60000);

	let r = wheel.remove(&Arc::new(new_index(expect_index[10].0, expect_index[10].1)));
	assert_eq!(r.time_point, 86400000);
}
