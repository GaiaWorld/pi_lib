/// Thread unsafe wheel structure, which supports quick deletion by index, with a precision of 10 milliseconds.
/// 
/// 
///
/// 
use std::cmp::{Ord, Ordering};
use std::mem::{replace, swap};
use std::fmt::{Debug, Formatter, Result as FResult};
use std::collections::VecDeque;

use heap::heap::Heap;
use dyn_uint::{UintFactory, ClassFactory};

// 描述了毫秒、秒、分、小时几种不同单位的任务在arr中的开始索引
static START:[u8; 4] = [0, 100, 160, 220];
// 描述了毫秒、秒、分、小时几种不同单位任务分别在arr中的个数
static CAPACITY:[u8; 4] = [100, 60, 60, 24];
// 描述了毫秒、秒、分、小时几种不同单位换算为毫秒后的数值
static UNIT:[u32; 4] = [10, 1000, 60000, 3600000];

pub struct Wheel<T>{
	// 毫秒精度为10， 秒，分钟， 小时精度为1, arr中，将容纳 100(单位：10毫秒)+60(秒)+60(分)+24(小时) 时间内的任务
	// 超出该时间的任务放入heap中
	arr: [Vec<(Item<T>, usize)>; 244],
	heap: Heap<Item<T>>,

	len: usize,

	// 为0毫秒的任务特殊优化
    zero_arr:Vec<(Item<T>, usize)>,
	zero_cache: Vec<(Item<T>, usize)>,
	
	// 记录每个不同时间单位当前的位置
	point:[u8; 4],
	time: u64,//当前时间

	// 弹出缓存，调用roll_once方法，使时间向前推动，超时的任务将缓存在这里
	pop_catch: VecDeque<(Item<T>, usize)>,
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
			time:0,
			pop_catch: VecDeque::new(),
			len: 0,
		}
	}

	pub fn len(&self) -> usize {
		self.len
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
	pub fn get_one_zero(&mut self) -> Option<(Item<T>, usize)> {
		if let Some(ts) = self.zero_arr.pop() {
			self.len -= 1;
			return Some(ts);
		}
		None
		// replace(&mut self.zero_arr, replace(&mut self.zero_cache, Vec::new()))
	}

    #[inline]
	pub fn get_zero(&mut self) -> Vec<(Item<T>, usize)>{
		let vec = replace(&mut self.zero_arr, replace(&mut self.zero_cache, Vec::new()));
		self.len -= vec.len();
		vec
	}

    #[inline]
    pub fn set_zero_cache(&mut self, v: Vec<(Item<T>, usize)>){
		replace(&mut self.zero_cache, v);
	}

	//插入元素
	pub fn insert< F: UintFactory + ClassFactory<usize>>(&mut self, item: Item<T>, index: usize, index_factory: &mut F){
		// 计算时间差
		let mut diff = sub(item.time_point, self.time);
		self.len += 1;
		// println!("diff============{}", diff);

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
		
		let layer = if diff < 1000{
			self.insert_ms((item, index), diff, index_factory);
			return;
		}else if diff < 61000{
			1
		}else if diff < 3661000{
			2
		}else{
			3
		};
		
		let mut pre_layer = 0;
		while pre_layer < layer{
			diff += self.point[pre_layer] as u64 * UNIT[pre_layer] as u64;
			pre_layer += 1;
		}
		self.insert_wheel((item, index), layer, diff, index_factory);
	}

	pub fn roll< F: UintFactory + ClassFactory<usize>>(&mut self, index_factory: &mut F) -> Vec<(Item<T>, usize)>{
		self.time += 10;
		let point = self.point[0] as usize;
		let s = START[0] as usize;
		let r= replace(&mut self.arr[point + s], Vec::new());
		self.point[0] = next_tail(point as u8, 1, CAPACITY[0]);
		// println!("roll===============time:{}, len:{}, self_len:{}", self.time, r.len(), self.len());
		if self.point[0] == 0 {
			self.adjust(1, index_factory);
		}
		
		self.len -= r.len();
		r
	}

	/// 时间向后推动10ms，并将超时任务缓存起来，外部需要通过pop放取出超时任务
	pub fn roll_once< F: UintFactory + ClassFactory<usize>>(&mut self, index_factory: &mut F){
		self.time += 10;
		let point = self.point[0] as usize;
		let s = START[0] as usize;

		// 将超时任务交换到出来
		let r = replace(&mut self.arr[point + s], Vec::new());
		for i in 0..r.len(){
			let (_, index) = r[i];
			index_factory.store(index, i);
			index_factory.set_class(index, 246);
		}

		let old_arr = replace(&mut self.pop_catch, VecDeque::from(r));
		replace(&mut self.arr[point + s], Vec::from(old_arr));

		self.point[0] = next_tail(point as u8, 1, CAPACITY[0]);
		if self.point[0] == 0 {
			self.adjust(1, index_factory);
		}
	}

	/// 调用roll_once后， 可调用本方法取出超时任务
	pub fn pop(&mut self) -> Option<(Item<T>, usize)>{

		let r = self.pop_catch.pop_front();
		if r.is_some() {
			self.len -= 1;
		}
		r

	}

	//Panics if index is out of bounds.
	pub fn delete< F: UintFactory + ClassFactory<usize>>(&mut self, class: usize, index:usize, index_factory: &mut F) -> Option<(Item<T>, usize)> {
		let r = if class == 245 { //heap的类型为245
			unsafe { Some(self.heap.delete(index, index_factory)) }
		} else if class == 244 {
            Wheel::delete_wheel(&mut self.zero_arr, index, index_factory)
        } else if class == 246{
			Wheel::delete_catch(&mut self.pop_catch, index, index_factory)
		}else {//wheel的类型为1
			Wheel::delete_wheel(&mut self.arr[class], index, index_factory)
		};
		if r.is_some() {
			self.len -= 1;
		}
		r
	}

	//clear all elem
	pub fn clear(&mut self){
		self.heap.clear();
		for v in self.arr.iter_mut() {
			v.clear();
		}
		self.zero_arr.clear();
		self.zero_cache.clear();
		self.pop_catch.clear();
		self.point = [0,0,0,0];
		self.time = 0;
		self.len = 0;
	}

	//插入到毫秒轮
	fn insert_ms< F: UintFactory + ClassFactory<usize>>(&mut self, item: (Item<T>, usize), diff: u64, index_factory: &mut F){
		let i = (next_tail(self.point[0], (diff/10) as u8, 100)) as usize;
		// println!("insert_ms===========diff:{}, i:{}", diff, i);
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

	fn delete_catch< F: UintFactory + ClassFactory<usize>>(arr: &mut VecDeque<(Item<T>, usize)>, index: usize, index_factory: &mut F) -> Option<(Item<T>, usize)> {
		if let Some(mut r) = arr.pop_back() {
			if index < arr.len(){
				index_factory.store(r.1, index);
				swap(&mut r, &mut arr[index]);
			}
			return Some(r);
		}
		None
	}

	/// 前进一个单位
	fn adjust< F: UintFactory + ClassFactory<usize>>(&mut self, layer: usize, index_factory: &mut F){
		// println!("adjust==============");
		if layer > 3 {
			self.adjust_heap(index_factory);
		} else {
			let point = self.point[layer] as usize;
			let s = START[layer] as usize;
			// println!("adjust==============layer:{}, point:{}, start:{}, time:{}", layer, point, s, self.time);
			let mut r = VecDeque::from(replace(&mut self.arr[point + s], Vec::new()));
			// println!("adjust==============point:{}, layer:{}", self.point[layer], layer);
			loop {
				match r.pop_front() {
					Some(v) => self.adjust_item(v, index_factory),
					None => break,
				}
			}
			replace(&mut self.arr[point + s], Vec::from(r));
			self.point[layer] = next_tail(point as u8, 1, CAPACITY[layer]);
			
			if self.point[layer] == 0 {
				self.adjust(layer + 1, index_factory);
			}
		}
	}

	fn adjust_heap< F: UintFactory + ClassFactory<usize>>(&mut self, index_factory: &mut F){
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
			let value = unsafe{self.heap.delete(0, index_factory)};
			self.adjust_item(value, index_factory);
		}
	}

	// 根据时间，调整任务位置
	fn adjust_item< F: UintFactory + ClassFactory<usize>>(&mut self, value: (Item<T>, usize), index_factory: &mut F) {
		let diff = match value.0.time_point > self.time{
			true => value.0.time_point - self.time - 1,
			false => 0,
		};
		// println!("adjust_item===========item_time:{}, self:time{}, diff:{}", value.0.time_point, self.time, diff );
		match diff{
			0..1000 => self.insert_ms(value, diff, index_factory),
			1000..61000 => {
				// println!("insert wheel===========diff:{}, layer:{}, UNIT:{}, point:{}, time:{}, time_point:{}", diff, 1, UNIT[1], self.point[1], self.time, value.0.time_point);
				self.insert_wheel(value, 1, diff, index_factory);
			},
			61000..3661000 => self.insert_wheel(value, 2, diff, index_factory),
			_ => self.insert_wheel(value, 3, diff, index_factory)
		}
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
	time:{},
	len:{},
)"##,
               arr_str,
               self.zero_arr,
               self.zero_cache,
               self.heap,
               self.point,
			   self.time,
			   self.len,
        )
    }
}

// 某个单位的指针的下一时刻位置
// cur_local: 当前位置， span： 跨度， capacity：当前指针的最大位置
#[inline]
fn next_tail(cur_local: u8, span: u8, capacity: u8) -> u8{
	(cur_local + span)%capacity
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
