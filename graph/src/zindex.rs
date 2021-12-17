use std::ops::Range;

use idtree::IdTree;
use map::vecmap::VecMap;

/// zindex系统
/// zindex的[min max), 采用Range, 开闭区间。
/// 设计分配如下： 如果父容器为 0 100.
/// 子节点为1个的话：Empty(1,33), Node(33,66), Empty(66,100). 
/// 为2个的话： Empty(1,20), Node(20,40), Empty(40,60), Node(60,80), Empty(80,100). 
/// 为3个的话： Empty(1,14), Node(14,28), Empty(28,42), Node(42,56), Empty(56,70), Node(70,84), Empty(84,100).
/// 采用整数， 有最小单元的限制。




#[derive(Debug, Clone, PartialEq)]
pub enum DirtyType {
  None,
  Normal, // 
  Recursive, // 递归脏，计算所有的子节点
}

#[derive(Debug, Clone)]
pub struct ZIndex {
  pub dirty: DirtyType, // 子节点设zindex时，将不是auto的父节点设脏
  pub z: Range<usize>, // 节点的z值范围，最小值也是节点自身的z值
  pub zdepth: usize, // 最后计算出来的z
}


  /// 整理方法
  fn calc(&mut self, idtree: &IdTree, zdepth: &mut MultiCaseImpl<Node, ZDepth>) {
    for (id, layer) in self.dirty.iter() {
      let (min_z, max_z, normal) = {
        let zi = unsafe {self.map.get_unchecked_mut(*id)};
        // println!("calc xxx: {:?} {:?}", id, zi);
        if zi.dirty == DirtyType::None {
          continue;
        }
        let b = zi.dirty == DirtyType::Normal;
        zi.dirty = DirtyType::None;
        zi.min_z = zi.pre_min_z;
        zi.max_z = zi.pre_max_z;
        (zi.min_z, zi.max_z, b)
      };
		let node = match idtree.get(*id) {
			Some(r) => if r.layer == layer {r} else {continue},
			None => continue,
		};
      // 设置 z_depth, 其他系统会监听该值
      unsafe {zdepth.get_unchecked_write(*id)}.set_0(min_z);
      //println!("zindex- calc: {:?} {:?} {:?} {:?}", id, min_z, max_z, normal);
      if node.count == 0 {
        continue;
      }
      self.cache.sort(&self.map, idtree, node.children.head, 0);
      if normal {
        self.cache.calc(&mut self.map, idtree, zdepth, min_z, max_z, node.count);
      }else{
        self.cache.recursive_calc(&mut self.map, idtree, zdepth, min_z, max_z, node.count);
      }
    }
    if self.dirty.count() > 0 {
      // 详细打印
      for (_id, n) in idtree.recursive_iter(2) {
        let mut v = String::new();
        for _ in 1..n.layer {
          v.push('-')
        }
        //println!("zindex- info: {:?} {:?} {:?} count:{:?}, layer:{:?}", v, id, unsafe {self.map.get_unchecked_mut(id)}, n.count, n.layer);
      }
    }
    self.dirty.clear();
  }


#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct ZSort (isize, usize, usize, usize); // (zindex, index, node_id, children_count)

// 计算z排序时使用的临时数据结构
struct Cache {
  node_heap: Vec<ZSort>,
  negative_heap: Vec<ZSort>,
  z_zero: Vec<ZSort>,
  z_auto: Vec<usize>,
  temp: Vec<(usize, f32, f32)>,
}
impl Cache {
  fn new() -> Cache {
    Cache {
      node_heap: Vec::new(),
      negative_heap: Vec::new(),
      z_zero: Vec::new(),
      z_auto: Vec::new(),
      temp: Vec::new(),
    }
  }

  // 循环计算子节点， 分类排序
  fn sort(&mut self, map: &VecMap<ZIndex>, idtree: &IdTree<()>, child: usize, mut order: usize) -> usize {
    // zindex为0或-1的不参与排序。 zindex排序。确定每个子节点的z范围。如果子节点的zindex==-1，则需要将其子节点纳入排序。
    for (id, n) in idtree.iter(child) {
      let zi = unsafe {map.get_unchecked(id)}.old;
      if zi == 0 {
          self.z_zero.push(ZSort(zi, order, id, n.count));
      }else if zi == -1 {
        self.z_auto.push(id);
        // 继续递归其子节点
        order = self.sort(map, idtree, n.children.head, order);
      }else if zi > 0 {
        self.node_heap.push(ZSort(zi, order, id, n.count));
      }else{
        self.negative_heap.push(ZSort(zi-1, order, id, n.count));
      }
      order+=1;
    }
    order
  }
  // 计算真正的z
  fn calc(&mut self, map: &mut VecMap<ZIndex>, idtree: &IdTree<()>, mut z: Range<usize>, count: usize) {
    min_z += 1.; // 第一个子节点的z，要在父节点z上加1
    let auto_len = self.z_auto.len();
    // println!("count--------------------------count: {}, auto_len: {}", count, auto_len);
    // 计算大致的劈分间距
    let split = if count > auto_len {
      (max_z - min_z - auto_len as f32) / (count - auto_len) as f32
    }else{
      1.
    };
    // println!("negative_heap: len: {:?}, value: {:?}", self.negative_heap.len(), self.negative_heap);
    while let Some(ZSort(_, _, n_id, c)) = self.negative_heap.pop() {
      max_z = min_z + split + split * c as f32;
      adjust(map, idtree, zdepth, n_id, unsafe {idtree.get_unchecked(n_id)}, min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
    }
    // println!("z_auto: len: {:?}, value: {:?}", self.z_auto.len(), self.z_auto);
    for n_id in &self.z_auto {
      adjust(map, idtree, zdepth, *n_id, unsafe {idtree.get_unchecked(*n_id)}, min_z, min_z, f32::NAN, 0.);
      min_z += 1.;
    }
    self.z_auto.clear();
    // println!("z_zero: len: {:?}, value: {:?}", self.z_zero.len(), self.z_zero);
    for &ZSort(_, _, n_id, c) in &self.z_zero {
      max_z = min_z + split + split * c as f32;
      adjust(map, idtree, zdepth, n_id, unsafe {idtree.get_unchecked(n_id)}, min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
    }
    self.z_zero.clear();
    // println!("z_node_heapzero: len: {:?}, value: {:?}", self.node_heap.len(), self.node_heap);
    while let Some(ZSort(_, _, n_id, c)) = self.node_heap.pop() {
      max_z = min_z + split + split * c as f32;
      adjust(map, idtree, zdepth, n_id, unsafe {idtree.get_unchecked(n_id)}, min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
    }
  }
// 计算真正的z
  fn recursive_calc(&mut self, map: &mut VecMap<ZIndex>, idtree: &IdTree, zdepth: &mut MultiCaseImpl<Node, ZDepth>, mut min_z: f32, mut max_z: f32, count: usize) {
    min_z += 1.; // 第一个子节点的z，要在父节点z上加1
    let auto_len = self.z_auto.len();
    // 计算大致的劈分间距
    let split = if count > auto_len {
      (max_z - min_z - auto_len as f32) / (count - auto_len) as f32
    }else{
      1.
    };
    let start = self.temp.len();
    while let Some(ZSort(_, _, n_id, c)) = self.negative_heap.pop() {
      max_z = min_z + split + split * c as f32;
      self.temp.push((n_id, min_z, max_z));
      min_z = max_z;
    }
    for n_id in &self.z_auto {
      self.temp.push((*n_id, min_z, min_z));
      min_z += 1.;
    }
    self.z_auto.clear();
    for &ZSort(_, _, n_id, c) in &self.z_zero {
      max_z = min_z + split + split * c as f32;
      self.temp.push((n_id, min_z, max_z));
      min_z = max_z;
    }
    self.z_zero.clear();
    while let Some(ZSort(_, _, n_id, c)) = self.node_heap.pop() {
      max_z = min_z + split + split * c as f32;
      self.temp.push((n_id, min_z, max_z));
      min_z = max_z;
    }
    while start < self.temp.len() {
      let (id, min_z, max_z) = self.temp.pop().unwrap();
      let zi = unsafe{map.get_unchecked_mut(id)};
      zi.dirty = DirtyType::None;
      zi.min_z = min_z;
      zi.pre_min_z = min_z;
      zi.max_z = max_z;
      zi.pre_max_z = max_z;
      // 设置 z_depth, 其他系统会监听该值
      unsafe {zdepth.get_unchecked_write(id)}.set_0(min_z);
      //println!("zindex- ----recursive_calc: {:?} {:?} {:?}", id, min_z, max_z);
      if min_z == max_z {
        continue
      }
      let node = unsafe {idtree.get_unchecked(id)};
      if node.count == 0 {
        continue;
      }
      self.sort(map, idtree, node.children.head, 0);
      //println!("zindex- ---recursive_sort: {:?} {:?} {:?}", id, node.children.head, node.count);
      self.recursive_calc(map, idtree, zdepth, min_z, max_z, node.count);
    }
  }
}
