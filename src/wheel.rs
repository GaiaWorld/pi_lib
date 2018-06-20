/**
 * 线程不安全的轮结构，支持根据索引快速删除
 */
use std::sync::atomic::{AtomicIsize, Ordering as AOrd};
use std::sync::Arc;
use std::cmp::{Ord, Ordering};
use std::mem::{uninitialized, replace, swap};

use heap::Heap;

static START:[u8; 4] = [0, 100, 160, 220];
static CAPACITY:[u8; 4] = [100, 60, 60, 24];
static UNIT:[u32; 4] = [10, 1000, 60000, 3600000];

pub struct Wheel<T>{
	arr: [Vec<(Item<T>, Arc<AtomicIsize>)>; 244],//毫秒精度为10， 秒，分钟， 小时精度为1
	heap: Heap<Item<T>>,
	point:[u8; 4],
	pub time: u64,//当前时间
}

impl<T: Clone> Wheel<T>{

	//创建一个轮， 支持四层轮
	pub fn new() -> Self{
		let mut arr: [Vec<(Item<T>, Arc<AtomicIsize>)>; 244] = unsafe{uninitialized()};
		for i in 0..244{
			arr[i] = Vec::new();
		}
		Wheel{
			arr: arr,
			heap: Heap::new(Ordering::Less),
			point:[0, 0, 0, 0],
			time:0
		}
	}

	//设置轮的时间
	pub fn set_time(&mut self, ms: u64){
		self.time = ms;
	}

	pub fn insert(&mut self, item: Item<T>) -> Arc<AtomicIsize>{
		let diff = match item.time_point > self.time {
			true => item.time_point - self.time,
			false => 0,
		};
		if diff >= 90061000{
			return self.heap.push(item);
		}
		let index = Arc::new(AtomicIsize::new(0));
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

	pub fn roll(&mut self) -> Vec<(Item<T>, Arc<AtomicIsize>)>{
		self.time += 10;
		self.forward(0)
	}

	pub fn try_remove(&mut self, index: Arc<AtomicIsize>) -> Option<Item<T>>{
		let i = index.load(AOrd::Relaxed);
		if i > 0{
			self.heap.try_remove(index)
		}else if i < 0{
			let index = split_index(i);
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
	pub fn remove(&mut self, index: Arc<AtomicIsize>) -> Item<T>{
		let i = index.load(AOrd::Relaxed);
		if i > 0{
			self.heap.remove(index)
		}else{//不是大于0， 则必须小于0， 否则Panic
			let index = split_index(i);
			Wheel::delete(&mut self.arr[index.0], index.1, i)
		}
	}

	//插入到毫秒轮
	fn insert_ms(&mut self, item: (Item<T>, Arc<AtomicIsize>), diff: u64){
		let i = (next_tail(0, (diff/10) as u8, 10)) as usize;
		item.1.store(sum_index(i, self.arr[i].len()), AOrd::Relaxed);
		self.arr[i].push(item);
	}

	//秒，分钟，小时轮的插入方法
	fn insert_wheel(&mut self, item: (Item<T>, Arc<AtomicIsize>), layer: usize, diff: u64){
		let i = (next_tail(0, (diff/(UNIT[layer] as u64)) as u8 - 1, CAPACITY[layer]) + START[layer]) as usize;
		item.1.store(sum_index(i, self.arr[i].len()), AOrd::Relaxed);
		self.arr[i].push(item);
	}

	fn delete(arr: &mut Vec<(Item<T>, Arc<AtomicIsize>)>, index: usize, i: isize) -> Item<T>{
		let mut r = arr.pop().unwrap();
		if index < arr.len(){
			r.1.store(i, AOrd::Relaxed);
			swap(&mut r, &mut arr[index]);
		}
		r.1.store(0, AOrd::Relaxed);
		r.0
	}

	//前进一个单位
	fn forward(&mut self, layer: usize) -> Vec<(Item<T>, Arc<AtomicIsize>)>{
		let point = self.point[layer] as usize;
		let s = START[layer] as usize;
		let r = replace(&mut self.arr[point + s], Vec::new());
		self.point[layer] = next_tail(point as u8, 1, (START[(layer + 1) as usize] - s as u8) as u8);
		if self.point[layer] == 0{
			let above = match layer > 3{
				true => self.get_from_heap(),
				false => self.forward(layer + 1)
			};
			if layer == 0 {
				for v in above.into_iter(){
					let d = sub(v.0.time_point, self.time);
					self.insert_ms(v, d)
				}
			}else {
				for v in above.into_iter() {
					let d = sub(v.0.time_point, self.time);
					self.insert_wheel(v, layer, d)
				}
			}
		}
		r
	}

	fn get_from_heap(&mut self) -> Vec<(Item<T>, Arc<AtomicIsize>)>{
		let mut r = Vec::new();
		let mut flag = true;
		while flag {
			match self.heap.get(0){
				Some(v) => {
					if sub(v.time_point, self.time) < 90061000{
						r.push(self.heap.get_top());
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

#[inline]
fn sum_index(index1: usize, index2: usize) -> isize{
	-((index2*244 + index1 + 1) as isize)
}

#[inline]
fn split_index(index: isize) -> (usize, usize){
	let index = (-index - 1) as usize;
	(index%244, index/244)
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
	let times = [0, 10, 1000, 3000, 3100, 50, 60000, 61000, 3600000, 3500000, 86400000, 86600000, 90061001];
	let expect_index = [-1, -2, -101, -103, -347, -6, -160, -161, -220, -218, -244, -488, 1];
	let mut i = 0;
	for v in times.iter(){
		let index = wheel.insert(Item{elem: v.clone(), time_point: v.clone() as u64});
		assert_eq!(&index.load(AOrd::Relaxed), &expect_index[i]);
		arr.push(index);
		i += 1;
	}

	let r = wheel.roll();
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 0);
	let r = wheel.roll();
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 10);

	for _i in 3..101{
		wheel.roll();
	}

	let r = wheel.roll();
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 1000);

	for _i in 2..201{
		wheel.roll();
	}

	let r = wheel.roll();
	assert_eq!(r.len(), 2);
	assert_eq!(r[0].0.time_point, 3000);

	let r = wheel.remove(arr[7].clone());
	assert_eq!(r.time_point, 61000);

	
	let r = wheel.remove(arr[6].clone());
	assert_eq!(r.time_point, 60000);

	let r = wheel.remove(arr[10].clone());
	assert_eq!(r.time_point, 86400000);
}
