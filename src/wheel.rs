/**
 * 线程不安全的轮结构，支持根据索引快速删除
 */
use std::sync::atomic::{AtomicIsize, Ordering as AOrd};
use std::sync::Arc;
use std::cmp::{Ord, Ordering};
use std::mem::{uninitialized, swap};

use heap::Heap;

pub struct Wheel<T>{
	pub arr: [Vec<(Item<T>, Arc<AtomicIsize>)>; 244],//毫秒精度为10， 秒，分钟， 小时精度为1
	heap: Heap<Item<T>>,
	start:[u8; 5],
	unit:[u32; 4],
	tail:[u8; 4],
	pub time: u64,//当前时间
}

impl<T: Clone + Ord> Wheel<T>{

	//创建一个轮， 支持四层轮
	pub fn new() -> Self{
		let mut arr: [Vec<(Item<T>, Arc<AtomicIsize>)>; 244] = unsafe{uninitialized()};
		for i in 0..244{
			arr[i] = Vec::new();
		}
		Wheel{
			arr: arr,
			heap: Heap::new(Ordering::Less),
			start:[0, 100, 160, 220, 244],
			unit:[10, 1000, 60000, 3600000],
			tail:[0, 0, 0, 0],
			time:0
		}
	}

	//设置轮的时间
	pub fn set_time(&mut self, ms: u64){
		self.time = ms;
	}

	pub fn insert(&mut self, item: Item<T>) -> Arc<AtomicIsize>{
		let diff = item.time_point - self.time;
		if diff >= 86400000{
			return self.heap.push(item);
		}
		let index = Arc::new(AtomicIsize::new(0));
		if diff < 1000{
			self.insert_wheel((item, index.clone()), 0, (diff + 10) as i64);
		}else if diff < 60000{
			self.insert_wheel((item, index.clone()), 1, diff as i64);
		}else if diff < 3600000{
			self.insert_wheel((item, index.clone()), 2, diff as i64);
		}else{
			self.insert_wheel((item, index.clone()), 3, diff as i64);
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
			if index.1 >= self.arr[index.0].len(){
				return None;
			}
			Some(Wheel::delete(&mut self.arr[index.0 as usize], index.1))
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
			Wheel::delete(&mut self.arr[index.0 as usize], index.1)
		}
	}

	fn insert_wheel(&mut self, item: (Item<T>, Arc<AtomicIsize>), layer: usize, mut diff: i64){
		if diff < 10{
			diff = 10;
		}
		let i = next_tail(0, (diff/(self.unit[layer] as i64)) as u8 - 1, self.start[layer + 1] - self.start[layer]) + self.start[layer];
		item.1.store(sum_index(i as usize, self.arr[i as usize].len()), AOrd::Relaxed);
		self.arr[i as usize].push(item);
	}

	fn delete(arr: &mut Vec<(Item<T>, Arc<AtomicIsize>)>, index: usize) -> Item<T>{
		let mut r = arr.pop().unwrap();
		if index < arr.len(){
			swap(&mut r, &mut arr[index])
		}
		r.1.store(0, AOrd::Relaxed);
		r.0
	}

	//前进一个单位
	fn forward(&mut self, layer: u8) -> Vec<(Item<T>, Arc<AtomicIsize>)>{
		let mut r = Vec::new();
		swap(&mut r, &mut self.arr[self.tail[layer as usize] as usize + (self.start[layer as usize]) as usize]);
		self.tail[layer as usize] = next_tail(self.tail[layer as usize], 1, (self.start[(layer + 1) as usize] - self.start[layer as usize]) as u8);
		if self.tail[layer as usize] == 0{
			let above = self.forward(layer + 1);
			for v in above.into_iter(){
				let mut diff = v.0.time_point - self.time;
				if layer == 0{
					diff += 10;
				}
				self.insert_wheel(v, layer as usize, diff as i64)
			}
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
fn next_tail(cur: u8, span: u8, capacity: u8) -> u8{
	(cur + span)%capacity
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
	let expect_index = [-1, -2, -101, -103, -347, -6, -161, -405, -221, -218, 1, 2];
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
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 3000);

	let r = wheel.remove(arr[4].clone());
	assert_eq!(r.time_point, 3100);

	let r = wheel.remove(arr[7].clone());
	assert_eq!(r.time_point, 61000);

	
	let r = wheel.remove(arr[6].clone());
	assert_eq!(r.time_point, 60000);

	let r = wheel.remove(arr[10].clone());
	assert_eq!(r.time_point, 86400000);
}
