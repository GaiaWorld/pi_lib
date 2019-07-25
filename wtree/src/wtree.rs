/**
 * 权重树，支持使用索引删除(实际上是一个堆的结构)
 */
use std::fmt::{Debug, Formatter, Result as FResult};
use std::mem::transmute_copy;
use std::ptr::write;
use std::ops::Drop;

use map::Map;

pub struct WeightTree<T, ID>(Vec<Item<T, ID>>);

impl<T, ID> Default for WeightTree<T, ID> {
    fn default() -> Self {
        WeightTree(Vec::new())
    }
}

impl<T, ID:Copy> WeightTree<T, ID> {

	//创建一颗权重树， 并初始容量
    #[inline]
	pub fn with_capacity(capacity: usize) -> Self{
		WeightTree(Vec::with_capacity(capacity))
	}

    #[inline]
    pub fn len(&self) -> usize{
		self.0.len()
	}

    #[inline]
	pub fn clear(&mut self) {
		self.0.clear();
	}

	//插入元素，返回该元素的位置
    #[inline]
	pub fn push<F:Map<Key=ID, Val=usize>>(&mut self, obj: T, weight: usize, obj_id: ID, id_map: &mut F){
		let len = self.0.len();
		self.0.push(Item{
			obj: obj,
			weight: weight,
			obj_id: obj_id,
			amount: weight,
		});
        id_map.insert(obj_id, len);
		self.up(len, id_map)
	}

	//All element weights and
    #[inline]
	pub fn amount(&self) -> usize{
		if self.0.len() == 0 {
			return 0
		}
		unsafe{self.0.get_unchecked(0)}.amount
	}

	//remove a element by weight and returns it, Panics if weight >= self.amount()
    #[inline]
	pub unsafe fn pop_unchecked<F:Map<Key=ID, Val=usize>>(&mut self, weight: usize, id_map: &mut F) -> (T, usize, ID){
	    self.delete(self.find(weight, 0, self.0.get_unchecked(0).weight), id_map)
	}

	//remove a element by weight, returns it, or None if weight >= self.amount()
    #[inline]
	pub fn pop<F:Map<Key=ID, Val=usize>>(&mut self, weight: usize, id_map: &mut F) -> Option<(T, usize, ID)>{
		if self.0.len() == 0 {
			return None
		}
		let r = unsafe{self.0.get_unchecked(0)};
		if r.amount <= weight {
			return None
		}
		let w = r.weight;
		Some(unsafe{self.delete(self.find(weight, 0, w), id_map)})
	}
    #[inline]
	pub fn get_by_weight(&self, weight: usize) -> Option<usize> {
		if self.0.len() == 0 {
			return None
		}
		let r = unsafe{self.0.get_unchecked(0)};
		if r.amount <= weight {
			return None
		}
		Some(self.find(weight, 0, r.weight))
	}

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> (&mut T, usize, ID){
		let r = self.0.get_unchecked_mut(index);
		(&mut r.obj, r.weight, r.obj_id)
	}

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> (&T, usize, ID){
		let r = self.0.get_unchecked(index);
		(&r.obj, r.weight, r.obj_id)
	}

    #[inline]
	pub unsafe fn get_unchecked_by_weight(&self, weight: usize) -> usize {
		self.find(weight, 0, self.0.get_unchecked(0).weight)
	}

    #[inline]
	pub unsafe fn update_weight<F:Map<Key=ID, Val=usize>>(&mut self, index: usize, weight: usize, id_map: &mut F){
		let r_index = self.up_update(index, weight, id_map);
		//如果没有上溯，则尝试下沉
		if r_index >= index{
			self.down(index, id_map);
		}
	}

    #[inline]
	pub unsafe fn delete<F:Map<Key=ID, Val=usize>>(&mut self, index: usize, id_map: &mut F) -> (T, usize, ID){
        let len = self.0.len();
		let (weight, weight_amount) = {
			let e = &self.0[index];
			(e.weight, e.amount)
		};
        
		// 优化算法： TODO
		if index + 1 < len {//如果需要移除的元素不是堆底元素， 需要将该元素位置设置为栈底元素并下沉
			let last = len - 1;
			let (last_weight, last_amount) = {
				let e = &self.0[last];
				(e.weight, e.amount)
			};
            
			self.0.swap(last, index);
			self.0[index].weight = weight;
			self.0[index].amount = weight_amount;
			self.0[last].weight = last_weight;
			self.0[last].amount = last_amount;
            id_map.insert(self.0[index].obj_id, index);
			self.up_update(index, last_weight, id_map);
			self.up_update(last, 0, id_map);
			self.down(index, id_map);
		}else{
			self.up_update(index, 0, id_map);
		}
		let elem = self.0.pop().unwrap();
		(elem.obj, weight, elem.obj_id)
	}

	//Finding element index according to weight
	#[inline]
	fn find(&self, mut weight: usize, cur_index:usize, cur_weight: usize) -> usize{
		//let cur_weight = unsafe{self.0.get_unchecked(cur_index)}.weight;
		if weight < cur_weight { //如果当前节点的权重比指定权重值大，应该直接返回该节点的索引
			return cur_index;
		}
		weight = weight - cur_weight;
		let left = (cur_index << 1) + 1;
		let item = unsafe{self.0.get_unchecked(left)};
		if item.amount <= weight { //比较左节点及其所有子节点权重和与指定权重的大小
			//如果指定权重更大， 则左节点及其所有子节点的权重都不可能超过指定权重， 从新计算指定权重， 在下一步从右节点中找节点
			return self.find(weight - item.amount, left + 1, unsafe{self.0.get_unchecked(left + 1)}.weight);//从右节点中找
		}
		return self.find(weight, left, item.weight)//如果指定权重更小，则可以从左节点中找到需要的元素
	}

	//上朔，更新当前节点和其父节点的权值  使用时应该保证index不会溢出
	fn up_update<F:Map<Key=ID, Val=usize>>(&mut self, mut cur: usize, weight: usize, id_map: &mut F) -> usize{
		let arr = &mut self.0;
		let old_weight = arr[cur].weight;
		{
			let elem = &mut arr[cur];
			elem.weight = weight;
			elem.amount = elem.amount - old_weight + weight;
		}
		if cur > 0{
			let mut parent = (cur - 1) >> 1;

            let mut elem: Item<T, ID> = unsafe{ transmute_copy(&arr[cur])};
			while weight > arr[parent].weight{
				let new_amount = elem.amount;
                id_map.insert(arr[parent].obj_id, cur);
				elem.amount = arr[parent].amount - old_weight + weight;
				arr[parent].amount = new_amount - elem.weight + arr[parent].weight;
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
				arr[i].amount = arr[i].amount + weight - old_weight;
			}
            id_map.insert(arr[cur].obj_id, cur);
		}
		cur
	}

	//上朔， 使用时应该保证index不会溢出
	fn up<F:Map<Key=ID, Val=usize>>(&mut self, mut cur: usize, id_map: &mut F){
		let arr = &mut self.0;
		if cur > 0{
			let mut parent = (cur - 1) >> 1;

            let mut elem: Item<T, ID> = unsafe{ transmute_copy(&arr[cur])};
            while elem.weight > arr[parent].weight {
                id_map.insert(arr[parent].obj_id, cur);
                let ew = elem.amount;
                elem.amount = arr[parent].amount + elem.weight;
                arr[parent].amount = ew + arr[parent].weight - elem.weight;
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

            let w = arr[cur].weight;
            id_map.insert(arr[cur].obj_id, cur);

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
	fn down<F:Map<Key=ID, Val=usize>>(&mut self, index: usize, id_map: &mut F) -> usize {
		// TODO 优化一下， 将[] 改为get_unchecked, 不要多次用arr[cur]这个的调用
		let mut cur = index;
		let arr = &mut self.0;
		let mut left = (cur << 1) + 1;
		let mut right = left + 1;
		let len = arr.len();
        let mut elem: Item<T, ID> = unsafe{ transmute_copy(&arr[cur])};
		while left < len {
			
			// 选择左右孩子的最较大值作为比较
			let mut child = left;
			if right < len && arr[right].weight > arr[left].weight {
				child = right;
			}
			//println!("left{}, len{}", left, len);
			match arr[cur].weight >= arr[child].weight{
				true => break,
				false => {
                    id_map.insert(arr[child].obj_id, cur);
					let cw = arr[child].amount;
					arr[child].amount = elem.amount;
					elem.amount = cw - arr[child].weight + elem.weight;
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
        id_map.insert(arr[cur].obj_id, cur);
		cur
	}
}

impl<T: Debug, ID: Debug> Debug for WeightTree<T, ID> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
               "WeightTree({:?})",
               self.0,
        )
    }
}

#[derive(Debug)]
pub struct Item<T, ID>{
    obj: T,
    weight: usize, //自身权重值
    obj_id: ID, //对象的id
    amount: usize, //自身权重值 和 子节点权重值的总和
}

impl<T, ID> Drop for WeightTree<T, ID> {
    fn drop(&mut self) {
        //println!("drop WeightTree----------");
    }
}