use std::fmt::{Debug, Formatter, Result as FResult};

use index_class::{IndexClassFactory};
use ver_index::VerIndex;

use wheel::{Wheel, Item};

pub struct SlabWheel<T, I:VerIndex> {
    factory: IndexClassFactory<usize,(), I>,
    wheel: Wheel<T, I>,
}
impl<T, I: VerIndex + Default> Default for SlabWheel<T, I> {
    fn default() -> Self {
        SlabWheel{
            factory: IndexClassFactory::default(),
            wheel: Wheel::new()
        }
    }
}
impl<T, I:VerIndex> SlabWheel<T, I>{

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
	pub fn insert(&mut self, elem: Item<T>) -> I::ID {
        let id = self.factory.create(0, 0, ());
		self.wheel.insert(elem, id, &mut self.factory);
        id
	}

	pub fn zero_size(&self) -> usize{
		self.wheel.zero_size()
	}

	pub fn get_zero(&mut self, vec: Vec<(Item<T>, I::ID)>) -> Vec<(Item<T>, I::ID)>{
		self.wheel.get_zero(vec)
	}

    pub fn replace_zero_cache(&mut self, vec: Vec<(Item<T>, I::ID)>) -> Vec<(Item<T>, I::ID)>{
        self.wheel.replace_zero_cache(vec)
	}

    //clear all elem
	pub fn clear(&mut self){
		self.wheel.clear();
	}

	pub fn roll(&mut self) -> Vec<(Item<T>, I::ID)>{
		self.wheel.roll(&mut self.factory)
	}

	pub fn remove(&mut self, id: I::ID) -> Option<Item<T>>{
		match self.factory.remove(id) {
            Some(i) => {
                let (elem, _) = self.wheel.delete(i.class, i.index, &mut self.factory);
                Some(elem)
            },
            None => None,
        }
	}

	//Panics if index is out of bounds.
	// pub fn remove(&mut self, index: usize) -> Item<T> {
	// 	let (elem, _) = self.wheel.delete(self.factory.get_class(index).clone(), self.factory.load(index), &mut self.factory);
    //     self.factory.destroy(index);
    //     elem
	// }
}

impl<T: Debug, I:VerIndex> Debug for SlabWheel<T, I> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
r##"Wheel( 
    factory: {:?},
    wheel: {:?},
)"##,
               self.factory,
               self.wheel
        )
    }
}


#[test]
fn test(){
    use ver_index::bit::BitIndex;
	let mut wheel:SlabWheel<u64, BitIndex> = SlabWheel::default();
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

	let r = wheel.remove(8).unwrap();
	assert_eq!(r.time_point, 61000);

	
	let r = wheel.remove(7).unwrap();
	assert_eq!(r.time_point, 60000);

	let r = wheel.remove(11).unwrap();
	assert_eq!(r.time_point, 86400000);

    println!("{:?}", wheel);
}
