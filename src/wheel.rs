/**
 * 线程不安全的轮结构，支持根据索引快速删除, 精度为10毫秒， 
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
    zero_arr:Vec<(Item<T>, Arc<AtomicIsize>)>,
    zero_cache: Vec<(Item<T>, Arc<AtomicIsize>)>,
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
			zero_arr: Vec::new(),
			zero_cache: Vec::new(),
			heap: Heap::new(Ordering::Less),
			point:[0, 0, 0, 0],
			time:0
		}
	}

	//设置轮的时间
	pub fn set_time(&mut self, ms: u64){
		self.time = ms;
	}

	//插入元素
	pub fn insert(&mut self, item: Item<T>) -> Arc<AtomicIsize>{
		// 计算时间差
		let mut diff = sub(item.time_point, self.time);

		//如果时间差为0， 则将其插入到zero_arr（特殊处理0毫秒）
		if diff == 0 {
			let index = Arc::new(AtomicIsize::new(sum_index(244, self.zero_arr.len())));
			self.zero_arr.push((item, index.clone()));
			return index;
		}
		if diff >= 90061000{
			return self.heap.push(item);
		}
		let index = Arc::new(AtomicIsize::new(0));
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

	pub fn get_zero(&mut self) -> Vec<(Item<T>, Arc<AtomicIsize>)>{
		replace(&mut self.zero_arr, replace(&mut self.zero_cache, Vec::new()))
	}

    pub fn set_zero_cache(&mut self, v: Vec<(Item<T>, Arc<AtomicIsize>)>){
		replace(&mut self.zero_cache, v);
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
		let i = (next_tail(self.point[0], (diff/10) as u8, 100)) as usize;
		item.1.store(sum_index(i, self.arr[i].len()), AOrd::Relaxed);
		self.arr[i].push(item);
	}

	//秒，分钟，小时轮的插入方法
	fn insert_wheel(&mut self, item: (Item<T>, Arc<AtomicIsize>), layer: usize, diff: u64){
		let i = (next_tail(self.point[layer], (diff/(UNIT[layer] as u64)) as u8 - 1, CAPACITY[layer]) + START[layer]) as usize;
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
	-((index2*245 + index1 + 1) as isize)
}

#[inline]
fn split_index(index: isize) -> (usize, usize){
	let index = (-index - 1) as usize;
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
	let times = [0, 10, 1000, 3000, 3100, 50, 60000, 61000, 3600000, 3500000, 86400000, 86600000, 90061001];
	let expect_index = [-245, -1, -100, -102, -103, -5, -159, -160, -219, -218, -243, -244, 1];
	let mut i = 0;
	for v in times.iter(){
		let index = wheel.insert(Item{elem: v.clone(), time_point: v.clone() as u64});
		assert_eq!(&index.load(AOrd::Relaxed), &expect_index[i]);
		arr.push(index);
		i += 1;
	}

	let r = wheel.roll();
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 10);

	for _i in 1..4{
		wheel.roll();
	}
	let r = wheel.roll();
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 50);

	for _i in 1..95{
		wheel.roll();
	}
	let r = wheel.roll();
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 1000);

	for _i in 1..200{
		wheel.roll();
	}
	let r = wheel.roll();
	assert_eq!(r.len(), 1);
	assert_eq!(r[0].0.time_point, 3000);

	let r = wheel.remove(arr[7].clone());
	assert_eq!(r.time_point, 61000);

	
	let r = wheel.remove(arr[6].clone());
	assert_eq!(r.time_point, 60000);

	let r = wheel.remove(arr[10].clone());
	assert_eq!(r.time_point, 86400000);
}
