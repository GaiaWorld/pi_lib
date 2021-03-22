//! 重定向映射表，一个使用usize作为key的映射表,
//! 使用usize作为key的映射表，`Vec`、`HashMap`都能做到。而`DenseVecMap`可以认为是这两种数据结构作为映射表时，
//! 在性能和内存占用上的一个平衡
//! 
//! 使用密集vec作为数据结构的Map
//! 该数据结构的实现，始于ecs系统的需求。
//! ecs的实体通常由一个数字表示，系统中的实体数字一般是从小到大的、连续的、小的数字
//!（就像Vec中元素的偏移，小是一个相对概念，Vec的偏移一般不会达到亿、兆的层次）。
//! 每个实体可以对应多个组件、为了追求同类组件内存空间的连续来提升性能，
//! ecs通常把不同实体实例拥有的相同类型的组件实例放在同一个连续的数据结构中,Vec就组件的容器的一个很好的选择。
//! 通过实体对应的数字，可以查询、删除、修改Vec中的组件（这个过程，实体数字作为偏移找到对应组件实例）。
//! Vec作为组件的容器时，如果所有实体或者多是实体都拥有这类组件，这将工作得很好，但是，假如我们有50个实体，仅仅0号实体和49号实体
//! 存在该组件，其它实体不需要该组件，那么可想而知，我们需要Vec的长度为50，在第0号位置和第49号位置存在一个组件，其他位置为空。
//! 这将大大浪费内存空间。一个替换的方案是使用HashMap作为该类组件的数据结构，但如果你追求性能，HashMap与Vec相比还是存在一定差距。
//! DenseVecMap为此提供一个折中的方案。其具有比HashMap更高的性能、比Vec更低的性能；比HashMap更高的内存浪费、比Vev更低的内存浪费
//! DenseVecMap使用两层数据结构来做到这一点。
//! 第一层为一个VecMap： 其长度与实体长度一样，其内容为一个usize，记录一个第二层数据结构的索引
//! 第二层为一个Vec：其长度该类组件的个数一样，其存放具体的组件数据。
//! 查询组件时，通过实体数字，在第一层找到该数字对应位置中存放的usize，在通过该usize找到第二层中的真实数据。
//! 删除组件时，如果组件在第二层Vec的中间位置，需要将Vec中最后一个组件放到该位置，并在第一层中改变对应的usize。
//! 因此,可以看出，DenseVecMap依然会浪费掉48个usize（第一层中1-48的位置）。当你的组件数据并量不大时，你并没有比Vec更节省内存空间
//! 比如，你的组件数据就是一个usize，你并没有节省内存，反而比Vec使用了更多的内存，而且相比Vec，性能也更低，此时，建议。
//! 尽管如此，在存储的单个数据体积稍大时，DenseVecMap依然是一个十分值得考虑的数据结构。
extern crate map;

use std::ops::{Index, IndexMut};
use std::mem::replace;

use map::Map;
use map::vecmap::VecMap;

/// 重定向映射表
#[derive(Default, Debug)]
pub struct DenseVecMap<T> {
    data_id: VecMap<usize>,
    data: Vec<T>,
    indexs: Vec<usize>,
}

impl<T> DenseVecMap<T> {
    /// 取到元素个数
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
	}
    
    /// 构造函数
	fn new() -> DenseVecMap<T> {
        DenseVecMap{
			data_id: VecMap::new(),
			data: Vec::new(),
			indexs: Vec::new(),
		}
    }

    /// 根据索引查询元素，如果不存在对应元素，返回None
    fn get(&self, id: usize) -> Option<&T> {
        match self.data_id.get(id) {
            Some(id) => Some(unsafe { self.data.get_unchecked(*id) } ),
            None => None,
        }  
    }

    /// 根据索引查询元素，如果不存在对应元素，返回None
    fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        match self.data_id.get(id) {
            Some(id) => Some(unsafe { self.data.get_unchecked_mut(*id) } ),
            None => None,
        }  
    }

    /// 根据索引移除元素并返回，如果不存在对应元素，返回None
    fn remove(&mut self, id: usize) -> Option<T> {
        match self.data_id.get(id) {
            Some(_) => Some(unsafe { self.remove_unchecked(id) } ),
            None => None,
        }  
    }

    /// 检查是否包含与一个索引相对应的元素
    fn contains(&self, id: usize) -> bool {
        match self.data_id.get(id) {
            Some(_) => true,
            None => false,
        }
    }

    /// 将索引和元素建立映射关系，并插入，如果该索引已经存在一个对应元素，则返回原有的元素
    fn insert(&mut self, id: usize, v: T) -> Option<T> {
        match self.data_id.get(id){
            Some(i) => {
                Some(replace(&mut self.data[*i], v))
            },
            None => {
                self.data_id.insert(id, self.data.len());
                self.data.push(v);
                self.indexs.push(id);
                None
            }
        }
    }

    /// 查询索引对应元素，如果不存在，将panic
    unsafe fn get_unchecked(&self, id: usize) -> &T {
        let did = *self.data_id.get_unchecked(id);
        self.data.get_unchecked(did)
    }

    /// 查询索引对应元素，如果不存在，将panic
    unsafe fn get_unchecked_mut(&mut self, id: usize) -> &mut T {
        let did = *self.data_id.get_unchecked(id);
        self.data.get_unchecked_mut(did)
    }

    /// 移除索引对应元素，如果不存在，将panic
    unsafe fn remove_unchecked(&mut self, id: usize) -> T {
        let did = *self.data_id.get_unchecked(id);
        let last = *self.indexs.last().unwrap();
        let r = self.data.swap_remove(did);
        self.indexs.swap_remove(did);
        
        let len = self.data.len();
        if len > 0 {
            self.data_id.replace(id, len);
            self.data_id.remove_unchecked(last);
        }else {
            self.data_id.remove_unchecked(id);
        }
        r
    }
}

/// DenseVecMap是一个映射表，为其实现Map trait
impl<T> Map for DenseVecMap<T> {
	type Key = usize;
	type Val = T;
    #[inline]
    fn get(&self, key: &usize) -> Option<&T> {
        self.get(*key)
    }

    #[inline]
    fn get_mut(&mut self, key: &usize) -> Option<&mut T> {
        self.get_mut(*key)
    }

    #[inline]
    unsafe fn get_unchecked(&self, key: &usize) -> &T {
        self.get_unchecked(*key)
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, key: &usize) -> &mut T {
        self.get_unchecked_mut(*key)
    }

    #[inline]
    unsafe fn remove_unchecked(&mut self, key: &usize) -> T {
        self.remove_unchecked(*key)
    }

    #[inline]
    fn insert(&mut self, key: usize, val: T) -> Option<T> {
        self.insert(key, val)
    }

    #[inline]
    fn remove(&mut self, key: &usize) -> Option<T> {
        self.remove(*key)
    }

    #[inline]
    fn contains(&self, key: &usize) -> bool {
        self.contains(*key)
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.data.capacity()
    }

    #[inline]
    fn mem_size(&self) -> usize {
        self.data_id.mem_size() + self.data.capacity() * std::mem::size_of::<T>() + self.indexs.capacity() * std::mem::size_of::<usize>()
	}
	fn with_capacity(_capacity: usize) -> Self {
		Self::new()
	}
}

impl<T> Index<usize> for DenseVecMap<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        let did = self.data_id[index];
        &self.data[did]
    }
}

impl<T> IndexMut<usize> for DenseVecMap<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        let did = self.data_id[index];
        &mut self.data[did]
    }
}


#[test]
fn test(){
    let mut vec = DenseVecMap::default();
    vec.insert(2, 2);
    vec.insert(10, 10);
    vec.insert(20, 20);

    println!("{:?}", vec);
    assert_eq!(2, *unsafe { vec.get_unchecked(2) });
    assert_eq!(10, *unsafe { vec.get_unchecked(10) });
    assert_eq!(20, *unsafe { vec.get_unchecked(20) });

    vec.remove(2);
    println!("{:?}", vec);
}