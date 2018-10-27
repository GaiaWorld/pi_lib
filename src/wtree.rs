/**
 * 权重树，支持使用索引删除
 */

//需要实现一个简单版的权重树， 不支持索引删除， 提高权重树的性能， TODO

use std::sync::atomic::{AtomicUsize, Ordering as AOrd};
use std::sync::Arc;

pub struct Item<T>{
    elem: T,
    count: usize, //自身权重值
    amount: usize, //自身权重值 和 子节点权重值的总和
    index: Arc<AtomicUsize>, //元素的位置
}

pub struct WeightTree<T>(Vec<Item<T>>);

impl<T> WeightTree<T> {

	//构建一颗权重树
	pub fn new() -> Self{
        WeightTree(Vec::new())
	}

	//创建一颗权重树， 并初始容量
	pub fn with_capacity(capacity: usize) -> Self{
		WeightTree(Vec::with_capacity(capacity))
	}

	//插入元素，返回该元素的位置
	pub fn push(&mut self, elem: T, weight: usize) -> Arc<AtomicUsize>{
		// println!("push------------------------------------{}", weight);
		// for i in 0..self.0.len(){
		// 	println!("update_weight----i:{}, a:{}, a.w:{}", i, self.0[i].amount, self.0[i].count,);
		// }
		let len = self.0.len();
		self.0.push(Item{
			elem: elem,
			count: weight,
			amount: weight,
			index: new_index(len + 1),
		});
		self.up(len)
	}

	//插入元素，返回该元素的位置
	pub fn push_with_index(&mut self, elem: T, weight: usize, index: &Arc<AtomicUsize>){
		// println!("update_weight----arr[0].a:{}, arr[1].a:{}, arr[0].w:{}, arr[1].w:{}, weight:{}", self.0[0].amount, self.0[1].amount, self.0[0].count, self.0[1].count,  weight);
		let len = self.0.len();
		store_index(len + 1, &index);
		self.0.push(Item{
			elem: elem,
			count: weight,
			amount: weight,
			index: index.clone(),
		});
		self.up(len);
	}

	//remove a element by index, Panics if index is out of bounds.
	pub fn remove(&mut self, index: &Arc<AtomicUsize>) -> T{
		println!("remove---------------------{}", index.load(AOrd::Relaxed));
		let i = load_index(index);
		let r = self.delete((i - 1) as usize, self.0.len());
		r.0
	}

	//remove a element by index; returns it, or None if it is not exist;
	pub fn try_remove(&mut self, index: &Arc<AtomicUsize>) -> Option<T>{
		let i = load_index(index);
		if i == 0{
			return None;
		}
		match self.try_delete((i - 1) as usize) {
			Some (v) => Some(v.0),
			None => None
		}
	}

	//All element weights and
	pub fn amount(&self) -> usize{
		match self.0.len(){
			0 => 0,
			_ => self.0[0].amount
		}
	}

	//remove a element by weight and returns it, Panics if weight >= self.amount()
	pub fn remove_by_weight(&mut self, weight: usize) -> (T, usize){
		// let mut r = Vec::new();
		// let mut r1 = Vec::new();

		// for i in 0..self.0.len(){
		// 	r.push(self.0[i].count);
		// 	r1.push(self.0[i].amount);
		// }
		// println!("count:{:?}",r);
		// println!("amount:{:?}",r1);
		let index = self.find(weight, 0);
		//println!("remove_by_weight----index:{}, weight:{}", index, weight);
		
		let r = self.delete(index, self.0.len());
		(r.0, r.1)
	}

	//remove a element by weight, returns it, or None if weight >= self.amount()
	pub fn try_remove_by_weight(&mut self, weight: usize) -> Option<(T, usize)>{
		let len = self.0.len();
		match len{
			0 => None,
			_ => {
				let all_w = self.0[0].amount;
				match all_w <= weight{
					true => None,
					false => {
						let index = self.find(weight, 0);
						let r = self.delete(index, self.0.len());
						Some((r.0, r.1))
					}
				}
			}
		}
	}

	pub fn get(&self, index: &Arc<AtomicUsize>) -> Option<&T>{
		let i = load_index(index);
		if i == 0 {
			return None;
		}
		match self.0.get(i - 1){
			Some(v) => Some(&v.elem),
			None => {None}
		}
	}

	pub fn get_mut(&mut self, index:  &Arc<AtomicUsize>) -> Option<&mut T>{
		let i = load_index(index);
		if i == 0 {
			return None;
		}
		match self.0.get_mut(i - 1){
			Some(v) => Some(&mut v.elem),
			None => {None}
		}
	}

	//get element by weight and returns its reference, Panics if weight >= self.amount()
	pub fn get_mut_by_weight(&mut self, weight: usize) -> (&mut T, &Arc<AtomicUsize>){
		let index = self.find(weight, 0);
		
		let e = &mut self.0[index];
		(&mut e.elem, &e.index)
	}

	//get element by weight and returns its reference, or None if weight >= self.amount()
	pub fn try_get_mut_by_weight(&mut self, weight: usize) -> Option<(&mut T, &Arc<AtomicUsize>)>{
		let len = self.0.len();
		match len{
			0 => None,
			_ => {
				let all_w = self.0[0].amount;
				match all_w <= weight{
					true => None,
					false => {
						let index = self.find(weight, 0);
						let e = &mut self.0[index];
						Some((&mut e.elem, &e.index))
					}
				}
			}
		}
	}

	pub fn update_weight(&mut self, weight: usize, index: &Arc<AtomicUsize>){
		let old_index = load_index(index);
		// println!("update_weight----arr[0].a:{}, arr[1].a:{}, arr[0].w:{}, arr[1].w:{}, index:{}, weight:{}", self.0[0].amount, self.0[1].amount, self.0[0].count, self.0[1].count, old_index, weight);
		let index = old_index - 1;
		let r_index = self.up_update(index, weight);

		//如果没有上溯，则尝试下沉
		match r_index < index{
			true => (),
			false => {self.down(index);}
		}
	}
	
	pub fn len(&self) -> usize{
		self.0.len()
	}

	pub fn clear(&mut self) {
		self.0.clear();
	}

	//Finding element index according to weight
	#[inline]
	fn find(&mut self, mut weight: usize, cur_index:usize) -> usize{
		let cur_weight = self.0[cur_index].count;
		//println!("cur_weight: {}, weight:{}", cur_weight, weight);
		match weight < cur_weight{
			true => {//如果当前节点的权重比指定权重值大，应该直接返回该节点的索引
				//println!("weight:{}, cur_weight:{}, cur_index:{}", weight, cur_weight, cur_index);
				return cur_index;
			},
			false => {//否则
				weight = weight - cur_weight;
				let left_index = (cur_index << 1) + 1;
				match self.0[left_index].amount <= weight{ //比较左节点及其所有子节点权重和与指定权重的大小
					true => {
						//如果指定权重更大， 则左节点及其所有子节点的权重都不可能超过指定权重， 从新计算指定权重， 在下一步从右节点中找节点
						weight = weight - self.0[left_index].amount;
						return self.find(weight, left_index + 1);//从右节点中找
					},
					false => return self.find(weight, left_index)//如果指定权重更小，则可以从左节点中找到需要的元素
				};
				
			}
		};
	}

	#[inline]
	fn delete(&mut self, index: usize, len: usize) -> (T, usize, Arc<AtomicUsize>){
		let (index_count, index_amount) = {
			let e = &self.0[index];
			(e.count, e.amount)
		};
		// 优化算法： TODO
		if index + 1 < len{//如果需要移除的元素不是堆底元素， 需要将该元素位置设置为栈底元素并下沉
			let last = len - 1;
			let (last_count, last_amount) = {
				let e = &self.0[last];
				(e.count, e.amount)
			};
			self.0.swap(last, index);
			self.0[index].count = index_count;
			self.0[index].amount = index_amount;
			self.0[last].count = last_count;
			self.0[last].amount = last_amount;
			store_index(index + 1, &self.0[index].index);
			self.up_update(index, last_count);
			self.up_update(last, 0);
			self.down(index);
		}else{
			self.up_update(index, 0);
		}
		let elem = self.0.pop().unwrap();
		store_index(0, &elem.index);
		(elem.elem, index_count, elem.index)
	}

	#[inline]
	fn try_delete(&mut self, index: usize) -> Option<(T, usize)>{
		let arr = &mut self.0;
		let len = arr.len();
		if index >= len {
			return None;
		}
		let r = self.delete(index, len);
		Some((r.0, r.1))
	}

	//上朔，更新当前节点和其父节点的权值  使用时应该保证index不会溢出
	fn up_update(&mut self, mut cur: usize, weight: usize) -> usize{
		let arr = &mut self.0;
		let old_count = arr[cur].count;
		//println!("up_update---cur:{}, weight:{}, count:{}, amount:{}", cur, weight, old_count, arr[cur].amount);
		{
			let elem = &mut arr[cur];
			elem.count = weight;
			elem.amount = elem.amount - old_count + weight;
		}
		if cur > 0{
			let mut parent = (cur - 1) >> 1;
			while weight > arr[parent].count{
				let new_amount = arr[cur].amount;
				store_index(cur + 1, &arr[parent].index);
				arr[cur].amount = arr[parent].amount - old_count + weight;
				//println!("up_update---------------parent{}, {},{},{}, {}",new_amount, arr[cur].count,arr[parent].count, cur, arr[cur].amount);
				arr[parent].amount = new_amount - arr[cur].count + arr[parent].count;
				arr.swap(cur, parent);
				
				// 往上迭代
				cur = parent;
				if parent == 0{
					break;
				}
				parent = (cur - 1) >> 1;
			}

			let mut i = cur;
			while i > 0{
				i = (i - 1) >> 1;//parent
				//println!("up_update1---i:{}, count:{}, amount:{}", i, arr[i].amount, arr[i].amount, );
				// if (arr[i].amount + weight) < old_count {
				// 	println!("up_update1---i:{}, count:{}, amount:{}, weight:{}", i, old_count, arr[i].amount, weight);
				// }
				arr[i].amount = arr[i].amount + weight - old_count;
			}
			store_index(cur + 1, &arr[cur].index);
		}
		cur
	}

	//上朔， 使用时应该保证index不会溢出
	fn up(&mut self, mut cur: usize) -> Arc<AtomicUsize>{
		let arr = &mut self.0;
		if cur > 0{
			let mut parent = (cur - 1) >> 1;
			while arr[cur].count > arr[parent].count{
				store_index(cur + 1, &arr[parent].index);
				let ew = arr[cur].amount;
				arr[cur].amount = arr[parent].amount + arr[cur].count;
				arr[parent].amount = ew + arr[parent].count - arr[cur].count;
				arr.swap(cur, parent);
				
				// 往上迭代
				cur = parent;
				if parent == 0{
					break;
				}
				parent = (cur - 1) >> 1;
			}

			let w = arr[cur].count;
			store_index(cur + 1, &arr[cur].index);

			let mut i = cur;
			while i > 0{
				i = (i - 1) >> 1;//parent
				arr[i].amount += w;
			}
		}
		arr[cur].index.clone()
	}

	/**
	 * 下沉
	 * Panics if index is out of bounds.
	 */
	fn down(&mut self, index: usize) -> usize {
		
		let mut cur = index;
		let arr = &mut self.0;
		let mut left = (cur << 1) + 1;
		let mut right = left + 1;
		let len = arr.len();
//println!("down------------index:{}, left{}, len{}", index, left, len);
		while left < len {
			
			// 选择左右孩子的最较大值作为比较
			let mut child = left;
			if right < len && arr[right].count > arr[left].count {
				child = right;
			}
			//println!("left{}, len{}", left, len);
			match arr[cur].count > arr[child].count{
				true => break,
				false => {
					store_index(cur + 1, &arr[child].index);
					let cw = arr[child].amount;
					arr[child].amount = arr[cur].amount;
					arr[cur].amount = cw - arr[child].count + arr[cur].count;
					arr.swap(cur, child);
					
					// 往下迭代
					cur = child;
					left = (cur << 1) + 1;
					right = left + 1;
				}
			}
		}
		store_index(cur + 1, &arr[cur].index);
		cur
	}
}

//new index in AtomicUsize, The last two bytes represent the type, and 2 means the wheel.
fn new_index(index: usize) -> Arc<AtomicUsize>{
	Arc::new(AtomicUsize::new((index << 2) + 2))
}

//store index in AtomicUsize, The last two bytes represent the type, and 2 means the wheel. 
#[inline]
fn store_index(index: usize, dst: &Arc<AtomicUsize>){
	dst.store((index << 2) + 2, AOrd::Relaxed);
}

//load index from AtomicUsize, The last two bytes represent the type, and 2 means the wheel.
#[inline]
fn load_index(index: &Arc<AtomicUsize>) -> usize{
	index.load(AOrd::Relaxed) >> 2
}

//判断一个index是否为另一个节点的父节点
// #[inline]
// fn assert(mut child: usize, parent: usize) -> bool{
// 	while child > 0 {
// 		child = (child - 1) >> 1;
// 		if child == parent {
// 			return true;
// 		}
// 	}
// 	false
// }

#[test]
fn test(){
	let mut wtree: WeightTree<u32> = WeightTree::new();
	wtree.push(100, 100);
	wtree.push(2000, 2000);
	wtree.push(50, 50);
	wtree.push(70, 70);
	wtree.push(500, 500);
	let index_2 = wtree.push(20, 20);
	assert_eq!(wtree.amount(), 2740);

	wtree.update_weight(60, &index_2);
	assert_eq!(load_index(&index_2), 3);
	assert_eq!(wtree.amount(), 2780);

	wtree.update_weight(20, &index_2);
	assert_eq!(wtree.amount(), 2740);
	assert_eq!(load_index(&index_2), 6);

	assert_eq!(wtree.remove_by_weight(2739).1, 20);
	assert_eq!(wtree.amount(), 2720);

	assert_eq!(wtree.remove_by_weight(2000).1, 500);
	assert_eq!(wtree.amount(), 2220);
	
	assert_eq!(wtree.remove_by_weight(1999).1, 2000);
	assert_eq!(wtree.amount(), 220);

	let index = wtree.push(30, 30);
	wtree.update_weight(80, &index);

	assert_eq!(wtree.remove_by_weight(140).1, 80);
	assert_eq!(wtree.amount(), 220);

}

#[cfg(test)]
use time::now_millis;
#[cfg(test)]
use rand::Rng;
#[cfg(test)]
use rand;
#[cfg(test)]
use std::collections::VecDeque;

#[test]
fn test_effic(){
	let mut weight_tree: WeightTree<u32> = WeightTree::new();
	let max = 100000;
	let now = now_millis();
	for i in 0..max{
		weight_tree.push(i, (i+1) as usize);
	}
	println!("push max_heap time{}",  now_millis() - now);

	let mut arr = VecDeque::new();
	let now = now_millis();
	for i in 0..max{
		arr.push_front(i);
	}
	println!("push VecDeque time{}",  now_millis() - now);

	let now = now_millis();
	for _ in 0..max{
		rand::thread_rng().gen_range(0, 100000);
	}
	println!("rand time{}",  now_millis() - now);


	let now = now_millis();
	for _ in 0..max{
		let r = rand::thread_rng().gen_range(0, weight_tree.amount());
		weight_tree.remove_by_weight(r);
	}
	println!("remove_by_weight time{}",  now_millis() - now);

	//let r = rand::thread_rng().gen_range(0, amount);
}

// #[test]
// fn test2(){
// 	let mut weight_tree: WeightTree<u32> = WeightTree::new();
// 	let r = [14, 9, 13, 6, 8, 10, 12, 0, 3, 2, 7, 1, 5, 4, 11];
// 	for i in 0..r.len(){
// 		weight_tree.push(r[i].clone(), r[i].clone() as usize);
// 	}

// 	weight_tree.remove_by_weight(45);
// 	weight_tree.remove_by_weight(10);
// }
