// use std::rc::{Rc, Weak};
// use std::cell::RefCell;
use std::clone::Clone;
use std::default::Default;
use std::ops::{Deref, DerefMut};
use std::rc::{Weak, Rc};
use std::cell::{RefCell};
use std::fmt;

use slab::{Slab, SlabIter, SlabIterMut};

use world::{ComponentMgr};

type ComponentIter<'a, T> = SlabIter<'a, ComponentP<T>>;
type ComponentIterMut<'a, T> = SlabIterMut<'a, ComponentP<T>>;

pub trait ComponentGroupTree {
    // type C: ComponentMgr;
    // fn new() -> Self;
}

pub trait ComponentHandler<T, C: ComponentMgr> {
    fn handle(&self, event: &Event, component_mgr: &mut C);
}

pub struct ComponentGroup<T: Sized, C: ComponentMgr>{
    components: Slab<ComponentP<T>>,
    handlers: Rc<RefCell<Vec<Weak<ComponentHandler<T, C>>>>>,
}

impl<T: Sized, C: ComponentMgr> Default for ComponentGroup<T, C> {
    fn default() -> Self{
        ComponentGroup {
            components: Slab::new(),
            handlers: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl<T: Sized + fmt::Debug, C: ComponentMgr> fmt::Debug for ComponentGroup<T, C>{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComponentGroup {{ components: {:?}, handlers_len: {} }}", self.components, self.handlers.borrow().len())
    }
}

impl<T: Sized, C: ComponentMgr> ComponentGroup<T, C>{
    pub fn new() -> Self{
        ComponentGroup{
            components: Slab::new(),
            handlers: Rc::new(RefCell::new(Vec::new())),
        }
    }

    // pub fn alloc(&mut self) -> &mut ComponentP<T>{
    //     let (id, value) = self.components.alloc();
    //     value
    // }

    pub fn insert(&mut self, component: T, parent: usize) -> usize{
        self.components.insert(ComponentP::new(component, parent))
    }

    pub fn try_remove(&mut self, id: usize) -> Option<ComponentP<T>>{
        if !self.components.contains(id){
            return None;
        }
        Some(self.components.remove(id))
    }

    pub fn remove(&mut self, id: usize) -> ComponentP<T> {
        self.components.remove(id)
    }

    pub fn try_get(&mut self, id: usize) -> Option<&ComponentP<T>>{
        self.components.get(id)
    }

    //这是一个非安全方法
    pub fn get(&self, id: usize) -> &ComponentP<T>{
        unsafe{ self.components.get_unchecked(id) }
    }

    pub fn try_get_mut(&mut self, id: usize) -> Option<&mut ComponentP<T>>{
        self.components.get_mut(id)
    }

    //这是一个非安全方法
    pub fn get_mut(&mut self, id: usize) -> &mut ComponentP<T>{
        unsafe{ self.components.get_unchecked_mut(id) }
    }

    pub fn iter_mut(&mut self) -> ComponentIterMut<T>{
        self.components.iter_mut()
    }

    pub fn iter(&self) -> ComponentIter<T>{
        self.components.iter()
    }

    //注册处理器
    pub fn register_handler(&self, monitor: Weak<ComponentHandler<T, C>>) {
        self.handlers.borrow_mut().push(monitor);
    }

    //取到处理器列表
    pub fn get_handlers(&self) -> Rc<RefCell<Vec<Weak<ComponentHandler<T, C>>>>> {
        self.handlers.clone()
    }

    //moitor的容器是一个Vec, 其移除的性能并不高， 如果需要频繁的移除， 考虑使用slab
    // pub fn unregister_moitor(&mut self, index: usize) -> Option<Box<Fn(EventType)>>{
    //     if index >= self.monitors.len(){
    //         None
    //     }else {
    //         Some(self.monitors.remove(index))
    //     }
    // }
}

//通知处理器
pub fn notify<T, C: ComponentMgr>(event: Event, handlers: &Vec<Weak<ComponentHandler<T, C>>>, mgr: &mut C) {
    for it in handlers.iter(){
        match Weak::upgrade(it) {
            Some(h) => {
                h.handle(&event, mgr);
            },
            None => println!("handler has been lost"),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Event<'a>  {
    ModifyField{
        id: usize,
        parent: usize,
        field: &'a str
    },

    ModifyIndex{
        id: usize,
        parent: usize,
        index: usize
    },

    ModifyFieldIndex{
        id: usize,
        parent: usize,
        field: &'a str,
        index: usize
    },

    Create{
        id: usize,
        parent: usize
    },

    Delete{
        id: usize,
        parent: usize
    }
}


#[derive(Clone, Default)]
pub struct ComponentP<C: Sized>{
    pub parent: usize,
    pub owner: C,
}

impl<C: fmt::Debug + Sized> fmt::Debug for ComponentP<C>{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComponentP {{ parent: {}, owner: {:?} }}", self.parent, self.owner)
    }
}

impl<C: Sized> ComponentP<C>{
    pub fn new(component: C, parent: usize) -> ComponentP<C>{
        ComponentP{
            parent: parent,
            owner: component,
        }
    }

    pub fn set_parent(&mut self, parent: usize){
        self.parent = parent;
    }

    pub fn get_parent(&mut self) -> usize{
        self.parent
    }

    pub fn unwrap(self) -> C{
        self.owner
    }
}

impl<C: Sized> Deref for ComponentP<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.owner
    }
}

impl<C: Sized> DerefMut for ComponentP<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.owner
    }
}

pub trait Builder<C:ComponentMgr, G, T>{
    fn build(self, group: &mut G) -> T;
}

// pub trait Create<M: ComponentMgr>{
//     type G;
//     fn create(group: &mut Self::G, parent: &usize) -> Self;
// }

// pub trait Destroy<M: ComponentMgr>{
//     type G;
//     fn destroy(group: &mut Self::G, id: &usize);
// }
