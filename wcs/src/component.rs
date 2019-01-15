// use std::rc::{Rc, Weak};
// use std::cell::RefCell;
use std::clone::Clone;
use std::ops::{Deref, DerefMut};
use std::rc::{Weak, Rc};
use std::cell::{RefCell};

use slab::{Slab, SlabIter, SlabIterMut};

use world::{ID, ComponentMgr};

type ComponentIter<'a, T> = SlabIter<'a, ComponentP<T>>;
type ComponentIterMut<'a, T> = SlabIterMut<'a, ComponentP<T>>;

pub trait ComponentGroupTree {
    type C: ComponentMgr;
    fn new() -> Self;
    // fn set_mgr(&mut self, mgr: Weak<RefCell<Self::C>>);
}

pub trait ComponentHandler<P: Point, C: ComponentMgr> {
    fn handle(&self, event: &Event<P>, component_mgr: &mut C);
}

pub struct ComponentGroup<T: Sized, P: Point, C: ComponentMgr>{
    components: Slab<ComponentP<T>>,
    handlers: Rc<RefCell<Vec<Weak<ComponentHandler<P, C>>>>>,
}

impl<T: Sized, P: Point, C: ComponentMgr> ComponentGroup<T, P, C>{
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

    pub fn insert(&mut self, component: T, parent: usize) -> P{
        let index = self.components.insert(ComponentP::new(component, parent));
        let mut point = P::default();
        point.set_id(index);
        point
    }

    pub fn try_remove(&mut self, id: &usize) -> Option<ComponentP<T>>{
        if !self.components.contains(id.clone()){
            return None;
        }
        Some(self.components.remove(id.clone()))
    }

    pub fn remove(&mut self, id: &usize) -> ComponentP<T> {
        self.components.remove(id.clone())
    }

    pub fn try_get(&mut self, id: &usize) -> Option<&ComponentP<T>>{
        self.components.get(id.clone())
    }

    //这是一个非安全方法
    pub fn get(&self, id: &usize) -> &ComponentP<T>{
        unsafe{ self.components.get_unchecked(id.clone()) }
    }

    pub fn try_get_mut(&mut self, id: &usize) -> Option<&mut ComponentP<T>>{
        self.components.get_mut(id.clone())
    }

    //这是一个非安全方法
    pub fn get_mut(&mut self, id: &usize) -> &mut ComponentP<T>{
        unsafe{ self.components.get_unchecked_mut(id.clone()) }
    }

    pub fn iter_mut(&mut self) -> ComponentIterMut<T>{
        self.components.iter_mut()
    }

    pub fn iter(&self) -> ComponentIter<T>{
        self.components.iter()
    }

    //注册处理器
    pub fn register_handler(&self, monitor: Weak<ComponentHandler<P, C>>) {
        self.handlers.borrow_mut().push(monitor);
    }

    //取到处理器列表
    pub fn get_handlers(&self) -> Rc<RefCell<Vec<Weak<ComponentHandler<P, C>>>>> {
        self.handlers.clone()
    }

    //moitor的容器是一个Vec, 其移除的性能并不高， 如果需要频繁的移除， 考虑使用slab
    // pub fn unregister_moitor(&mut self, index: usize) -> Option<Box<Fn(EventType<P>)>>{
    //     if index >= self.monitors.len(){
    //         None
    //     }else {
    //         Some(self.monitors.remove(index))
    //     }
    // }
}

//通知处理器
pub fn notify<P: Point, C: ComponentMgr>(event: Event<P>, handlers: &Vec<Weak<ComponentHandler<P, C>>>, mgr: &mut C) {
    for it in handlers.iter(){
        match Weak::upgrade(it) {
            Some(h) => {
                h.handle(&event, mgr);
            },
            None => println!("handler has been lost"),
        }
    }
}

pub enum Event<'a, P: Point>  {
    ModifyField{
        point: P,
        parent: usize,
        field: &'a str
    },
    ModifyIndex{
        point: P,
        parent: usize,
        index: usize
    },
    ModifyFieldIndex{
        point: P,
        parent: usize,
        field: &'a str,
        index: usize
    },
    Create{
        point: P,
        parent: usize
    },
    Delete{
        point: P,
        parent: usize
    }
}

// pub trait Group<C: Component, P: Point>{
//     fn set_group(&mut self, group: WeakComponentGroup<C, P>);
// }

pub trait Point: ID + Clone + Default + Sized{

}

#[derive(Clone, Default)]
pub struct PPoint<P: Point>{
    pub id: P,
    pub parent: usize,
}

impl<P: Point> Deref for PPoint<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl<P: Point> DerefMut for PPoint<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.id
    }
}

#[derive(Clone, Default)]
pub struct ComponentP<C: Sized>{
    pub parent: usize,
    pub owner: C,
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

// pub trait Create<M: ComponentMgr>{
//     type G;
//     fn create(group: &mut Self::G, parent: &usize) -> Self;
// }

// pub trait Destroy<M: ComponentMgr>{
//     type G;
//     fn destroy(group: &mut Self::G, id: &usize);
// }