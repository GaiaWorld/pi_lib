
use std::fmt::{Debug, Formatter, Result as FResult};

use dyn_uint::{UintFactory, ClassFactory, SlabFactory};
use crate::wheel::{Wheel as W, Item};

pub struct Wheel<T> {
    index_factory: SlabFactory<usize,()>,
    wheel: W<T>,
}

impl<T> Wheel<T>{

	//Create a wheel to support four rounds.
	pub fn new() -> Self{
		Wheel{
            index_factory: SlabFactory::new(),
            wheel: W::new()
        }
	}

	pub fn len(&self) -> usize {
		self.wheel.len()
	}

	//Setting wheel time
    #[inline]
	pub fn set_time(&mut self, ms: u64){
		self.wheel.set_time(ms);
	}

    #[inline]
    pub fn get_time(&mut self) -> u64{
		self.wheel.get_time()
	}

	//插入元素
	pub fn insert(&mut self, elem: Item<T>) -> usize{
        let index = self.index_factory.create(0, 0, ());
		self.wheel.insert(elem, index, &mut self.index_factory);
        index
	}

	pub fn zero_size(&self) -> usize{
		self.wheel.zero_size()
	}

	#[inline]
	pub fn get_one_zero(&mut self) -> Option<Item<T>> {
		if let Some((elem, index)) = self.wheel.get_one_zero() {
			self.index_factory.destroy(index);
			return Some(elem);
		}
		None
	}
	pub fn get_zero(&mut self) -> Vec<(Item<T>, usize)>{
		let arr = self.wheel.get_zero();
		for r in arr.iter() {
			self.index_factory.destroy(r.1);
		}
		arr
	}

    pub fn set_zero_cache(&mut self, v: Vec<(Item<T>, usize)>){
        self.wheel.set_zero_cache(v);
	}

    //clear all elem
	pub fn clear(&mut self){
		self.wheel.clear();
		self.index_factory.clear();
	}

	pub fn roll(&mut self) -> Vec<(Item<T>, usize)>{
		let arr = self.wheel.roll(&mut self.index_factory);
		for r in arr.iter() {
			self.index_factory.destroy(r.1);
		}
		arr
	}

	#[inline]
	pub fn roll_once(&mut self) {
		self.wheel.roll_once(&mut self.index_factory);
	}

	#[inline]
	pub fn pop(&mut self) -> Option<Item<T>> {
		if let Some((elem, index)) = self.wheel.pop() {
			self.index_factory.destroy(index);
			return Some(elem);
		}
		None
	}

	pub fn try_remove(&mut self, index: usize) -> Option<Item<T>>{
		match self.index_factory.try_load(index) {
            Some(i) => {
                if let Some((elem, _)) = self.wheel.delete(self.index_factory.get_class(index).clone(), i, &mut self.index_factory) {
					self.index_factory.destroy(index);
					return Some(elem);
				}

				None
            },
            None => None,
        }
	}

	//Panics if index is out of bounds.
	pub fn remove(&mut self, index: usize) -> Option<Item<T>> {
		if let Some((elem, _)) = self.wheel.delete(self.index_factory.get_class(index).clone(), self.index_factory.load(index), &mut self.index_factory) {
			self.index_factory.destroy(index);
			return Some(elem);
		}
		None
	}
}

impl<T: Debug> Debug for Wheel<T> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
r##"Wheel( 
    index_factory: {:?},
    wheel: {:?},
)"##,
               self.index_factory,
               self.wheel
        )
    }
}


#[test]
fn test(){
	let mut wheel = Wheel::new();
	let times = [0, 10, 1000, 3000, 3100, 50, 60000, 61000, 3600000, 3500000, 86400000, 86600000];
	//测试插入到轮中的元素位置是否正确
	for v in times.iter(){
		wheel.insert(Item{elem: v.clone(), time_point: v.clone() as u64});
	}

	//测试插入到堆中的元素位置是否正确
	let heap_elem = 90061001;
	wheel.insert(Item{elem: heap_elem, time_point: heap_elem as u64});

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

	let r = wheel.remove(8);
	assert_eq!(r.unwrap().time_point, 61000);

	
	let r = wheel.remove(7);
	assert_eq!(r.unwrap().time_point, 60000);

	let r = wheel.remove(11);
	assert_eq!(r.unwrap().time_point, 86400000);

    println!("{:?}", wheel);
}


#[test]
fn test1(){
	let mut wheel = Wheel::new();
	for i in 1..1001 {
		wheel.insert(Item{elem: i, time_point: 3000 });
	}

	let mut count = 0;
	let mut roll_count = 0;
	loop {
		match wheel.pop() {
			Some(_) => {
				count +=1;
				continue;
			},
			None => (),
		}
		wheel.roll_once();
		roll_count += 1;
		if roll_count == 3002 {
			println!("count: {}", count);
			break;
		}
	}
}



#[test]
fn test_insert() {

    let mut wheel = Wheel::new();

	let mut timer_refs = vec![];
    for i in 1..50001 {
        timer_refs.push(wheel.insert(Item{elem: i, time_point: 3000 }));
    }

	println!("timer refs = {:?}, len:{}", timer_refs,timer_refs.len());
}
