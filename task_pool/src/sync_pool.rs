use std::marker::Send;
use std::fmt;

use wtree::wtree::WeightTree;

use slab::Slab;
use index_class::{IndexClassFactory};
use ver_index::VerIndex;
use deque::deque::{Deque, Node, Direction};

use enums:: {DequeStat, FreeSign};

pub struct SyncPool<T: 'static, I:VerIndex>{
    slab: Slab<Node<T, I::ID>, I>,
    weight_queues: WeightTree<(), I::ID>,
    factory: IndexClassFactory<DequeStat, WeightQueue<T, I>, I>,
    len: usize,
}

unsafe impl<T: Send, I:VerIndex + Send> Send for SyncPool<T, I> {}
unsafe impl<T: Send, I:VerIndex + Send> Sync for SyncPool<T, I> {}

impl<T, I:VerIndex + Default> Default for SyncPool<T, I> {
    #[inline]
    fn default() -> Self {
        SyncPool {
            slab: Slab::default(),
            weight_queues: WeightTree::default(),
            factory: IndexClassFactory::default(),
            len: 0,
        }
    }
}
impl<T: 'static, I:VerIndex> SyncPool<T, I>{

    //任务数量
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn queue_len(&self) -> usize {
        self.weight_queues.len()
    }

    //create queues, if id is exist, return false
    #[inline]
    pub fn create_queue(&mut self, weight: usize) -> I::ID {
        self.factory.create(0, DequeStat::HalfLocked, WeightQueue::new(weight))
    }

    #[inline]
    pub fn remove_queue(&mut self, queue_id: I::ID) -> bool {
        match self.factory.remove(queue_id) {
            Some(mut q) => {
                q.value.queue.clear(&mut self.slab); // 将队列中的任务删除
                match q.class {
                    DequeStat::Normal =>{
                        unsafe {self.weight_queues.delete(q.index, &mut self.factory);};
                        true
                    },
                    _ => false
                }
            },
            _ => false
        }
    }

    #[inline]
    pub fn get_queue_stat(&self, queue_id: I::ID) -> Option<DequeStat> {
        match self.factory.get(queue_id) {
            Some(q) => Some(q.class.clone()),
            None => None,
        }
    }

    #[inline]
    pub fn lock_queue(&mut self, queue_id: I::ID)-> bool {
        match self.factory.get_mut(queue_id) {
            Some(q) => match q.class {
                DequeStat::Normal => {
                    q.class = DequeStat::Locked;
                    self.len -= q.value.len();
                    unsafe {self.weight_queues.delete(q.index, &mut self.factory)};
                    true
                },
                DequeStat::HalfLocked => {
                    q.class = DequeStat::Locked;
                    self.len -= q.value.len();
                    true
                },
                _ => true
            },
            None => false
        }
    }

    #[inline]
    pub fn unlock_queue(&mut self, queue_id: I::ID) -> FreeSign{
        let weight = match self.factory.get_mut(queue_id) {
            Some(ref mut q) => match q.class {
                DequeStat::Locked => {
                    if q.value.len() == 0 {
                        q.class = DequeStat::HalfLocked;
                        return FreeSign::Success;
                    }
                    self.len += q.value.len();
                    q.class = DequeStat::Normal;
                    q.value.get_weight()
                }
                _ => return FreeSign::Ignore,
            },
            _ => return FreeSign::Error,
        };
        self.weight_queues.push((), weight, queue_id, &mut self.factory);
        FreeSign::Success
    }
    // 创建一个任务的id
    pub fn create_node(&mut self, task: T) -> I::ID {
        self.slab.insert(Node::new(task, I::ID::default(), I::ID::default()))
    }
    // 设置指定id的节点
    pub fn get_node_mut(&mut self, id: I::ID) -> Option<&mut Node<T, I::ID>> {
        self.slab.get_mut(id)
    }
    //Find a queue with weight, Removes the first element from the queue and returns it, Painc if weight >= get_weight().
    #[inline]
    pub fn pop(&mut self, weight: usize) -> (Option<T>, I::ID) {
        match self.weight_queues.get_by_weight(weight) {
            Some(index) => {
                let queue_id = unsafe {self.weight_queues.get_unchecked(index)}.2;
                let q = unsafe { self.factory.get_unchecked_mut(queue_id) };
                let r = q.value.pop(&mut self.slab);
                unsafe{ self.weight_queues.update_weight(q.index, q.value.get_weight(), &mut self.factory)};
                self.len -= 1;
                (r, queue_id)
            },
            _ => (None, I::ID::default())
        }
    }

    //pop elements from specified queue, and not update weight, Painc if weight >= get_weight()
    #[inline]
    pub fn pop_lock(&mut self, weight: usize) -> (Option<T>, I::ID) {
        match self.weight_queues.pop(weight, &mut self.factory) {
            Some((_, _, queue_id)) => {
                let q = unsafe { self.factory.get_unchecked_mut(queue_id) };
                self.len -= q.value.len();
                q.class = DequeStat::Locked;
                (q.value.pop(&mut self.slab), queue_id)
            },
            _ => (None, I::ID::default())
        }
    }
    //Append task id to the queue of the specified ID. return locked
    #[inline]
    pub fn push_id(&mut self, queue_id: I::ID, task_id: I::ID, direct: Direction) -> bool {
        match self.factory.get_mut(queue_id) {
            Some(q) =>{
                q.value.push_id(task_id, direct, &mut self.slab);
                self.len += 1;
                 match q.class {
                    DequeStat::Locked => true,
                    DequeStat::HalfLocked => {
                        q.class = DequeStat::Normal;
                        self.weight_queues.push((), q.value.get_weight(), queue_id, &mut self.factory);
                        false
                    },
                    DequeStat::Normal => {
                        unsafe{self.weight_queues.update_weight(q.index, q.value.get_weight(), &mut self.factory)};
                        false
                    }
                }
            },
            _ => true
        }
    }
    //Append an element to the queue of the specified ID. return task id and locked
    #[inline]
    pub fn push(&mut self, queue_id: I::ID, task: T, direct: Direction) -> (I::ID, bool) {
        match self.factory.get_mut(queue_id) {
            Some(q) =>{
                let id = q.value.push(task, direct, &mut self.slab);
                self.len += 1;
                 match q.class {
                    DequeStat::Locked => {
                        (id, true)
                    },
                    DequeStat::HalfLocked => {
                        q.class = DequeStat::Normal;
                        self.weight_queues.push((), q.value.get_weight(), queue_id, &mut self.factory);
                        (id, false)
                    },
                    DequeStat::Normal => {
                        unsafe{self.weight_queues.update_weight(q.index, q.value.get_weight(), &mut self.factory)};
                        (id, false)
                    }
                }
            },
            _ => (I::ID::default(), true)
        }
    }

    //取队列的权重（所有任务的权重总值)
    #[inline]
    pub fn get_weight(&self) -> usize{
        self.weight_queues.amount()
    }

    #[inline]
    pub fn remove(&mut self, queue_id: I::ID, id: I::ID) -> Option<T> {
        match self.factory.get_mut(queue_id) {
            Some(ref mut q) => {
                let r = q.value.remove(id, &mut self.slab);
                if r.is_none() {
                    return None
                }
                match q.class {
                    DequeStat::Normal => {
                        let weight = q.value.get_weight();
                        if weight == 0 { //如果权重为0， 应该从权重树中删除
                            q.class = DequeStat::HalfLocked;
                            unsafe{ self.weight_queues.delete(q.index, &mut self.factory) };
                        }else {
                            unsafe{ self.weight_queues.update_weight(q.index, weight, &mut self.factory) }
                        }
                    },
                    _ => ()
                }
                self.len -= 1;
                r
            },
            None => None,
        }
    }

    //清空同步任务池
    #[inline]
    pub fn clear(&mut self) {
        self.weight_queues.clear();
        self.factory.clear();
        self.len = 0;
    }


}

impl<T: fmt::Debug, I: VerIndex + fmt::Debug> fmt::Debug for SyncPool<T, I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r##"SyncPool (
weight_queues: {:?},
factory: {:?}
len: {},
        )"##, self.weight_queues, self.factory, self.len)
    }
}


//可删除的权重队列， WeightQueue(权重, 队列)
pub struct WeightQueue<T, I: VerIndex>{
    weight_unit: usize, //单个任务权重
    queue: Deque<T, Slab<Node<T, I::ID>, I>, I::ID>, //队列
}

impl<T, I: VerIndex> WeightQueue<T, I>{

    #[inline]
    fn new(weight_unit: usize) -> Self{
        WeightQueue{
            weight_unit: weight_unit,
            queue: Deque::default(),
        }
    }

    #[inline]
    fn pop(&mut self, slab: &mut Slab<Node<T, I::ID>, I>) -> Option<T> {
        self.queue.pop_front(slab)
    }
    #[inline]
    fn push_id(&mut self, task_id: I::ID , direct: Direction, slab: &mut Slab<Node<T, I::ID>, I>) {
        self.queue.push_id(task_id, direct, slab)
    }

    #[inline]
    fn push(&mut self, task: T, direct: Direction, slab: &mut Slab<Node<T, I::ID>, I>) -> I::ID {
        self.queue.push(task, direct, slab)
    }

    #[inline]
    fn remove(&mut self, id: I::ID, slab: &mut Slab<Node<T, I::ID>, I>) -> Option<T>{
        self.queue.remove(id, slab)
    }

    //取队列的权重（所有任务的权重总值）
    #[inline]
    fn get_weight(&self) -> usize {
        let len = self.queue.len();
        self.weight_unit * len
    }

    #[inline]
    fn len(&self) -> usize{
       self.queue.len()
    }
}

impl<T, I: VerIndex> Drop for WeightQueue<T, I> {
    fn drop(&mut self) {
        //println!("drop Queue----------");
    }
}

impl<T: fmt::Debug, I: VerIndex + fmt::Debug> fmt::Debug for WeightQueue<T, I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r##"WeightQueue {{
weight_unit: {:?},
queue: {:?}
    }}"##, self.weight_unit, self.queue)
    }
}

// #[test]
// fn test(){
// 	let mut pool: SyncPool<u32> = SyncPool::new();
    
//     // for i in 1..66{
//     //     pool.create_queue(1);
//     // }
//     // let queue1 = pool.create_queue(1);
//     // let queue2 = pool.create_queue(2);
//     // pool.try_remove_queue(queue2);
// }