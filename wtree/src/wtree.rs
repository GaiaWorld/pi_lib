//! 权重树的核心逻辑
//! 提供插入、删除、弹出的等主要接口
//! 另外，使用了UintFactory，关于UintFactory，你可以看看其作用https://github.com/GaiaWorld/pi_lib/tree/master/dyn_uint
use std::fmt::{Debug, Formatter, Result as FResult};
use std::mem::transmute_copy;
use std::ptr::write;
use std::ops::Drop;

use dyn_uint::{UintFactory};

/// 权重树
pub struct WeightTree<T>(Vec<Item<T>>);

impl<T> WeightTree<T> {

	/// 构建一颗权重树
    #[inline]
	pub fn new() -> Self{
        WeightTree(Vec::new())
	}

	/// 创建一颗权重树， 并初始容量
    #[inline]
	pub fn with_capacity(capacity: usize) -> Self{
		WeightTree(Vec::with_capacity(capacity))
	}

	/// 权重树的长度
    #[inline]
    pub fn len(&self) -> usize{
		self.0.len()
	}

    #[inline]
	pub fn clear(&mut self) {
		self.0.clear();
	}

	/// 插入元素，返回该元素的位置
    #[inline]
	pub fn push<F:UintFactory>(&mut self, elem: T, weight: usize, index: usize, index_factory: &mut F){
		let len = self.0.len();
		self.0.push(Item{
			elem: elem,
			count: weight,
			amount: weight,
			index: index,
		});
        index_factory.store(index, len);
		self.up(len, index_factory)
	}

	/// 取到总权重
    #[inline]
	pub fn amount(&self) -> usize{
		match self.0.len(){
			0 => 0,
			_ => self.0[0].amount
		}
	}

	/// 指定一个权重，弹出对应任务
    #[inline]
	pub unsafe fn pop<F:UintFactory>(&mut self, weight: usize, index_factory: &mut F) -> (T, usize, usize){
		let index = self.find(weight, 0);
	    self.delete(index, index_factory)
	}

	/// 指定一个权重，尝试弹出一个对应任务，如果指定权重大于权重树中任务的总权重，返回None
    #[inline]
	pub fn try_pop<F:UintFactory>(&mut self, weight: usize, index_factory: &mut F) -> Option<(T, usize, usize)>{
		match self.0.len(){
			0 => None,
			_ => match self.0[0].amount <= weight{
					true => None,
					false => {
                        let index = self.find(weight, 0);
                        Some(unsafe{self.delete(index, index_factory)})
                    }
				}
		}
	}

	/// 根据索引取到对应任务的可变引用，如果不存在，将panic
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T{
		&mut self.0.get_unchecked_mut(index).elem
	}


	/// 根据索引取到对应任务的不可变引用，如果不存在，将panic
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T{
		&self.0.get_unchecked(index).elem
	}

	/// 指定一个权重，根据权重查找可以被该权重弹出的任务，但不弹出，仅返回不可变引用
    #[inline]
	pub unsafe fn get_unchecked_by_weight(&self, weight: usize) -> (&T, usize) {
		let index = self.find(weight, 0);
        (&self.0[index].elem, self.0[index].index)
	}

	/// 指定一个权重，根据权重查找可以被该权重弹出的任务，但不弹出，仅返回可变引用
    #[inline]
	pub unsafe fn get_unchecked_mut_by_weight(&mut self, weight: usize) -> (&mut T, usize) {
		let index = self.find(weight, 0);
        let i = self.0[index].index.clone();
        (&mut self.0[index].elem, i)
	}

	/// 根据索引，重新指定其对应任务的权重
    #[inline]
	pub unsafe fn update_weight<F:UintFactory>(&mut self, weight: usize, index: usize, index_factory: &mut F){
		let r_index = self.up_update(index, weight, index_factory);
		//如果没有上溯，则尝试下沉
		if r_index >= index{
			self.down(index, index_factory);
		}
	}

	/// 删除指定索引对应的权重，返回被删除的任务，如果不存在，将panic
    #[inline]
	pub unsafe fn delete<F:UintFactory>(&mut self, index: usize, index_factory: &mut F) -> (T, usize, usize){
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
            index_factory.store(self.0[index].index, index);
			self.up_update(index, last_count, index_factory);
			self.up_update(last, 0, index_factory);
			self.down(index, index_factory);
		}else{
			self.up_update(index, 0, index_factory);
		}
		let elem = self.0.pop().unwrap();
        //index_factory.store(elem.index, 0);
		(elem.elem, index_count, elem.index)
	}

	// 更具权重查找可被弹出的任务
	#[inline]
	fn find(&self, mut weight: usize, cur_index:usize) -> usize{
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

	//上朔，更新当前节点和其父节点的权值  使用时应该保证index不会溢出
	fn up_update<F:UintFactory>(&mut self, mut cur: usize, weight: usize, index_factory: &mut F) -> usize{
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
                index_factory.store(arr[parent].index, cur);
				elem.amount = arr[parent].amount - old_count + weight;
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
            //arr[cur] = elem;

			let mut i = cur;
			while i > 0{
				i = (i - 1) >> 1;//parent
				arr[i].amount = arr[i].amount + weight - old_count;
			}
            index_factory.store(arr[cur].index, cur);
		}
		cur
	}

	//上朔， 使用时应该保证index不会溢出
	fn up<F:UintFactory>(&mut self, mut cur: usize, index_factory: &mut F){
		let arr = &mut self.0;
		if cur > 0{
			let mut parent = (cur - 1) >> 1;

            let mut elem: Item<T> = unsafe{ transmute_copy(&arr[cur])};
            while elem.count > arr[parent].count {
                index_factory.store(arr[parent].index, cur);
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
            index_factory.store(arr[cur].index, cur);

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
	fn down<F:UintFactory>(&mut self, index: usize, index_factory: &mut F) -> usize {
		
		let mut cur = index;
		let arr = &mut self.0;
		let mut left = (cur << 1) + 1;
		let mut right = left + 1;
		let len = arr.len();
//println!("down------------index:{}, left{}, len{}", index, left, len);
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
                    index_factory.store(arr[child].index, cur);
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
        //arr[cur] = elem;
        index_factory.store(arr[cur].index, cur);
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

/// 权重树中每节点记录的数据
#[derive(Debug)]
pub struct Item<T>{
    elem: T,
    count: usize, //自身权重值
    amount: usize, //自身权重值 和 子节点权重值的总和
    index: usize, //元素的位置
}

impl<T> Drop for WeightTree<T> {
    fn drop(&mut self) {
        //println!("drop WeightTree----------");
    }
}