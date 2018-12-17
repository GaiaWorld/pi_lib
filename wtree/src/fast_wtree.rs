/**
 * 一个更快速的权重树，不支持索引删除
 */
use std::fmt::{Debug, Formatter, Result as FResult};
use std::mem::transmute_copy;
use std::ptr::write;

#[derive(Debug)]
pub struct Item<T>{
    elem: T,
    count: usize, //自身权重值
    amount: usize, //自身权重值 和 子节点权重值的总和
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

    //All element weights and
	pub fn amount(&self) -> usize{
		match self.0.len(){
			0 => 0,
			_ => self.0[0].amount
		}
	}

    pub fn len(&self) -> usize{
		self.0.len()
	}

	pub fn clear(&mut self) {
		self.0.clear();
	}

	//插入元素，返回该元素的位置
	pub fn push(&mut self, elem: T, weight: usize){
		let len = self.0.len();
		self.0.push(Item{
			elem: elem,
			count: weight,
			amount: weight,
		});
		self.up(len)
	}

	//remove a element by weight and returns it, Panics if weight >= self.amount()
	pub fn pop(&mut self, weight: usize) -> (T, usize){
		let index = self.find(weight, 0);
	    self.delete(index)
	}

	//remove a element by weight, returns it, or None if weight >= self.amount()
	pub fn try_pop(&mut self, weight: usize) -> Option<(T, usize)>{
		match self.0.len(){
			0 => None,
			_ => match self.0[0].amount <= weight{
					true => None,
					false => {
                        let index = self.find(weight, 0);
                        Some(self.delete(index))
                    }
				}
		}
	}

	//Finding element index according to weight
	#[inline]
	fn find(&mut self, mut weight: usize, cur_index:usize) -> usize{
		let cur_weight = self.0[cur_index].count;
		match weight < cur_weight{
			true => {//如果当前节点的权重比指定权重值大，应该直接返回该节点的索引
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
	fn delete(&mut self, index: usize) -> (T, usize){
        let len = self.0.len();
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
			self.up_update(index, last_count);
			self.up_update(last, 0);
			self.down(index);
		}else{
			self.up_update(index, 0);
		}
		let elem = self.0.pop().unwrap();
		(elem.elem, index_count)
	}

	//上朔，更新当前节点和其父节点的权值  使用时应该保证index不会溢出
	fn up_update(&mut self, mut cur: usize, weight: usize) -> usize{
		let arr = &mut self.0;
		let old_count = arr[cur].count;
		{
			let elem = &mut arr[cur];
			elem.count = weight;
			elem.amount = elem.amount - old_count + weight;
		}
		if cur > 0{
			let mut parent = (cur - 1) >> 1;

            let mut elem: Item<T> = unsafe{ transmute_copy(&arr[cur])};
			while weight > arr[parent].count{
				let new_amount = elem.amount;
				elem.amount = arr[parent].amount - old_count + weight;
				//println!("up_update---------------parent{}, {},{},{}, {}",new_amount, arr[cur].count,arr[parent].count, cur, arr[cur].amount);
				arr[parent].amount = new_amount - elem.count + arr[parent].count;
                let src = arr.as_mut_ptr();
                unsafe{src.wrapping_offset(parent as isize).copy_to(src.wrapping_offset(cur as isize), 1)};
				
				// 往上迭代
				cur = parent;
				if parent == 0{
					break;
				}
				parent = (cur - 1) >> 1;
			}
            unsafe{write(arr.as_mut_ptr().wrapping_offset(cur as isize), elem)};

			let mut i = cur;
			while i > 0{
				i = (i - 1) >> 1;//parent
				//println!("up_update1---i:{}, count:{}, amount:{}", i, arr[i].amount, arr[i].amount, );
				// if (arr[i].amount + weight) < old_count {
				// 	println!("up_update1---i:{}, count:{}, amount:{}, weight:{}", i, old_count, arr[i].amount, weight);
				// }
				arr[i].amount = arr[i].amount + weight - old_count;
			}
		}
		cur
	}

	//上朔， 使用时应该保证index不会溢出
	fn up(&mut self, mut cur: usize){
		let arr = &mut self.0;
		if cur > 0{
			let mut parent = (cur - 1) >> 1;

            let mut elem: Item<T> = unsafe{ transmute_copy(&arr[cur])};
            while elem.count > arr[parent].count {
                let ew = elem.amount;
                elem.amount = arr[parent].amount + elem.count;
                arr[parent].amount = ew + arr[parent].count - elem.count;
                let src = arr.as_mut_ptr();
                unsafe{src.wrapping_offset(parent as isize).copy_to(src.wrapping_offset(cur as isize), 1)};
                
                // 往上迭代
                cur = parent;
                if parent == 0{
                    break;
                }
                parent = (cur - 1) >> 1;
            }
            unsafe{write(arr.as_mut_ptr().wrapping_offset(cur as isize), elem)};

            let w = arr[cur].count;
			let mut i = cur;
			while i > 0{
				i = (i - 1) >> 1;//parent
				arr[i].amount += w;
			}
		}
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
        let mut elem: Item<T> = unsafe{ transmute_copy(&arr[cur])};
		while left < len {
			
			// 选择左右孩子的最较大值作为比较
			let mut child = left;
			if right < len && arr[right].count > arr[left].count {
				child = right;
			}
			//println!("left{}, len{}", left, len);
			match arr[cur].count >= arr[child].count{
				true => break,
				false => {
					let cw = arr[child].amount;
					arr[child].amount = elem.amount;
					elem.amount = cw - arr[child].count + elem.count;
                    let src = arr.as_mut_ptr();
                    unsafe{src.wrapping_offset(child as isize).copy_to(src.wrapping_offset(cur as isize), 1)};
					// 往下迭代
					cur = child;
					left = (cur << 1) + 1;
					right = left + 1;
				}
			}
		}
        unsafe{write(arr.as_mut_ptr().wrapping_offset(cur as isize), elem)};
		cur
	}
}

impl<T: Debug> Debug for WeightTree<T> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
               "WeightTree({:?})",
               self.0,
        )
    }
}

#[cfg(test)]
use time::now_millis;
#[cfg(test)]
use rand::Rng;

#[test]
fn test_effic(){
	let mut weight_tree: WeightTree<u32> = WeightTree::new();
	let max = 100000;
	let now = now_millis();
	for i in 0..max{
		weight_tree.push(i, (i+1) as usize);
	}
	println!("fast_wtree: push max_heap time{}",  now_millis() - now);

	let now = now_millis();
	for _ in 0..max{
		rand::thread_rng().gen_range(0, 100000);
	}
	println!("fast_wtree: rand time{}",  now_millis() - now);


	let now = now_millis();
	for _ in 0..max{
		let r = rand::thread_rng().gen_range(0, weight_tree.amount());
		weight_tree.pop(r);
	}
	println!("fast_wtree: remove_by_weight time{}",  now_millis() - now);

	//let r = rand::thread_rng().gen_range(0, amount);
}

