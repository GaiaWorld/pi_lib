// use std::rc::{Rc, Weak};
// use std::cell::RefCell;
use std::clone::Clone;
use std::ops::{Deref, DerefMut};
use std::rc::{Weak};
use std::cell::RefCell;

use slab::{Slab, SlabIter, SlabIterMut};

use world::{ID, ComponentMgr};

type ComponentIter<'a, T> = SlabIter<'a, ComponentP<T>>;
type ComponentIterMut<'a, T> = SlabIterMut<'a, ComponentP<T>>;

pub trait ComponentGroupTree {
    type C: ComponentMgr;
    fn new() -> Self;
    fn set_mgr(&mut self, mgr: Weak<RefCell<Self::C>>);
}

pub trait ComponentHandler<P: Point, C: ComponentMgr> {
    fn handle(&self, event: EventType<P>, component_mgr: &mut C);
}

pub struct ComponentGroup<T: Sized, P: Point, C: ComponentMgr>{
    components: Slab<ComponentP<T>>,
    handlers: Vec<Weak<ComponentHandler<P, C>>>,
    mgr: Weak<RefCell<C>>,
}

impl<T: Sized, P: Point, C: ComponentMgr> ComponentGroup<T, P, C>{
    pub fn new() -> Self{
        ComponentGroup{
            components: Slab::new(),
            handlers: Vec::new(),
            mgr: Weak::new(),
        }
    }

    pub fn set_mgr(&mut self, mgr: Weak<RefCell<C>>){
        self.mgr = mgr;
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

    pub fn try_remove(&mut self, point: &P) -> Option<T>{
        let id = point.id();
        if !self.components.contains(id){
            return None;
        }
        Some(self.components.remove(id).unwrap())
    }

    pub fn remove(&mut self, id: &P) -> T {
        self.components.remove(id.id()).unwrap()
    }

    pub fn try_get(&mut self, id: &P) -> Option<&ComponentP<T>>{
        self.components.get(id.id())
    }

    //这是一个非安全方法
    pub fn get(&self, id: &P) -> &ComponentP<T>{
        unsafe{ self.components.get_unchecked(id.id()) }
    }

    pub fn try_get_mut(&mut self, id: &P) -> Option<&mut ComponentP<T>>{
        self.components.get_mut(id.id())
    }

    //这是一个非安全方法
    pub fn get_mut(&mut self, id: &P) -> &mut ComponentP<T>{
        unsafe{ self.components.get_unchecked_mut(id.id()) }
    }

    pub fn iter_mut(&mut self) -> ComponentIterMut<T>{
        self.components.iter_mut()
    }

    pub fn iter(&self) -> ComponentIter<T>{
        self.components.iter()
    }

    pub fn register_handlers(&mut self, monitor: Weak<ComponentHandler<P, C>>) {
        self.handlers.push(monitor);
    }

    //moitor的容器是一个Vec, 其移除的性能并不高， 如果需要频繁的移除， 考虑使用slab
    // pub fn unregister_moitor(&mut self, index: usize) -> Option<Box<Fn(EventType<P>)>>{
    //     if index >= self.monitors.len(){
    //         None
    //     }else {
    //         Some(self.monitors.remove(index))
    //     }
    // }

    pub fn notify(&self, event: EventType<P>){
        let mgr = match Weak::upgrade(&self.mgr){
            Some(m) => m,
            None => {
                println!("ComponentMgr has been lost");
                return;
            },
        };
        let m_borrow = unsafe{&mut(*mgr.as_ptr())};
        for it in self.handlers.iter(){
            match Weak::upgrade(it) {
                Some(h) => {
                    h.handle(event.clone(), &mut *m_borrow);
                },
                None => println!("handler has been lost"),
            }
        }
    }
}

pub enum EventType<'a, P: Point>  {
    ModifyField(P, &'a str),
    ModifyIndex(P, usize),
    ModifyFieldIndex(P, &'a str, usize),
    Create(P),
    Delete(P)
}

impl<'a, P: Point> Clone for EventType<'a, P>{
    fn clone (&self) -> Self {
        match self {
            EventType::ModifyField(t, ref s) => EventType::ModifyField(t.clone(), s),
            EventType::ModifyIndex(t, ref u) => EventType::ModifyIndex(t.clone(), u.clone()),
            EventType::ModifyFieldIndex(t, ref s, u) => EventType::ModifyFieldIndex(t.clone(), s, u.clone()) ,
            EventType::Create(t) => EventType::Create(t.clone()),
            EventType::Delete(t) => EventType::Delete(t.clone())
        }
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