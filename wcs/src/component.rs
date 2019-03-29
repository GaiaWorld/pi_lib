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

pub struct Handlers<T: Sized, E, C: ComponentMgr>(Rc<RefCell<Vec<Weak<ComponentHandler<T, E, C>>>>>);

impl<T: Sized, E, C: ComponentMgr> Handlers<T, E, C> {
    pub fn new() -> Self {
        Handlers(Rc::new(RefCell::new(Vec::new())))
    }

    //注册处理器
    pub fn register_handler(&self, monitor: Weak<ComponentHandler<T, E, C>>) {
        self.0.borrow_mut().push(monitor);
    }

    //取到处理器列表
    pub fn get_handlers(&self) -> Rc<RefCell<Vec<Weak<ComponentHandler<T, E, C>>>>> {
        self.0.clone()
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn notify(&self, event: E, mgr: &mut C) {
        let handlers = self.0.borrow();
        for it in handlers.iter(){
            match Weak::upgrade(it) {
                Some(h) => {
                    h.handle(&event, mgr);
                },
                None => println!("handler has been lost"),
            }
        }
    }
}

impl<T: Sized, E, C: ComponentMgr> Default for Handlers<T, E, C> {
    fn default() -> Self{
        Handlers::new()
    }
}

impl<T: Sized, E, C: ComponentMgr> Clone for Handlers<T, E, C> {
    fn clone(&self) -> Self{
        Handlers(self.0.clone())
    }
}

pub struct GenHandlers<T: Sized, C: ComponentMgr>{
    modify_field: Handlers<T, ModifyFieldEvent, C>,
    delete: Handlers<T, DeleteEvent, C>,
    create: Handlers<T, CreateEvent, C>,
}

impl<T: Sized, C: ComponentMgr> Clone for GenHandlers<T, C> {
    fn clone(&self) -> Self{
        GenHandlers{
            modify_field: self.modify_field.clone(),
            delete: self.delete.clone(),
            create: self.create.clone(),
        }
    }
}

impl<T: Sized, C: ComponentMgr> Default for GenHandlers<T, C> {
    fn default() -> Self{
        GenHandlers::new()
    }
}

impl<T: Sized, C: ComponentMgr> GenHandlers<T, C> {
    pub fn new() -> Self {
        GenHandlers{
            modify_field: Handlers::new(),
            delete: Handlers::new(),
            create: Handlers::new(),
        }
    }

    //注册处理器
    pub fn register_modify_field_handler(&self, monitor: Weak<ComponentHandler<T, ModifyFieldEvent, C>>) {
        self.modify_field.register_handler(monitor);
    }

    //注册处理器
    pub fn register_delete_handler(&self, monitor: Weak<ComponentHandler<T, DeleteEvent, C>>) {
        self.delete.register_handler(monitor);
    }

    //注册处理器
    pub fn register_create_handler(&self, monitor: Weak<ComponentHandler<T, CreateEvent, C>>) {
        self.create.register_handler(monitor);
    }

    pub fn len(&self) -> usize {
        self.modify_field.len() + self.delete.len() + self.create.len()
    }

    pub fn notify_modify_field(&self, event: ModifyFieldEvent, mgr: &mut C){
        self.modify_field.notify(event, mgr);
    }

    pub fn notify_create(&self, event: CreateEvent, mgr: &mut C){
        self.create.notify(event, mgr);
    }

    pub fn notify_delete(&self, event: DeleteEvent, mgr: &mut C){
        self.delete.notify(event, mgr);
    }
}

pub struct DeleteEvent{
    pub parent: usize,
    pub id: usize,
}

pub struct CreateEvent{
    pub parent: usize,
    pub id: usize,
}

pub struct ModifyFieldEvent{
    pub parent: usize,
    pub id: usize,
    pub field: &'static str,
}

// pub struct ModifyIndexEvent<'a>{
//     parent: usize,
//     id: usize,
//     index: &'a str,
// }

pub trait ComponentHandler<T, E, C: ComponentMgr> {
    fn handle(&self, event: &E, component_mgr: &mut C);
}

pub struct ComponentGroup<T: Sized, C: ComponentMgr>{
    components: Slab<ComponentP<T>>,
    handlers: GenHandlers<T, C>,
}

impl<T: Sized, C: ComponentMgr> Default for ComponentGroup<T, C> {
    fn default() -> Self{
        ComponentGroup::new()
    }
}

impl< T: Sized + fmt::Debug, C: ComponentMgr> fmt::Debug for ComponentGroup<T, C>{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComponentGroup {{ components: {:?}, handlers_len: {} }}", self.components, self.handlers.len())
    }
}

impl<T: Sized, C: ComponentMgr> ComponentGroup<T, C>{
    pub fn new() -> Self{
        ComponentGroup{
            components: Slab::new(),
            handlers: GenHandlers::new(),
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
    pub fn register_modify_field_handler(&self, monitor: Weak<ComponentHandler<T, ModifyFieldEvent, C>>) {
        self.handlers.register_modify_field_handler(monitor);
    }

    pub fn register_create_handler(&self, monitor: Weak<ComponentHandler<T, CreateEvent, C>>) {
        self.handlers.register_create_handler(monitor);
    }

    pub fn register_delete_handler(&self, monitor: Weak<ComponentHandler<T, DeleteEvent, C>>) {
        self.handlers.register_delete_handler(monitor);
    }

    //取到处理器列表
    pub fn get_handlers(&self) -> GenHandlers<T, C> {
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

pub struct SingleModifyEvent {
    pub field: &'static str,
}

pub struct SingleCase<T, C: ComponentMgr>{
    value: T,
    handlers: Handlers<T, SingleModifyEvent, C>, //单例组件只有修改事件
}

impl<T, C: ComponentMgr> SingleCase<T, C> {
    pub fn new(value: T) -> Self{
        SingleCase{
            value,
            handlers: Handlers::default(),
        }
    }

    pub fn get_handlers(&self) -> Handlers<T, SingleModifyEvent, C> {
        self.handlers.clone()
    }
}

impl<T, C: ComponentMgr> Deref for SingleCase<T, C> {
    type Target=T;
    fn deref(&self) -> &Self::Target{
        &self.value
    }
}

impl<T, C: ComponentMgr> DerefMut for SingleCase<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target{
        &mut self.value
    }
}

pub struct SingleCaseWriteRef<'a, T, C: ComponentMgr> {
    mgr: usize,
    value: &'a mut SingleCase<T, C>,
}

impl<'a, T, C: ComponentMgr> Deref for SingleCaseWriteRef<'a, T, C> {
    type Target=SingleCase<T, C>;
    fn deref(&self) -> &Self::Target{
        &self.value
    }
}

impl<'a, T, C: ComponentMgr> DerefMut for SingleCaseWriteRef<'a, T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target{
        &mut self.value
    }
}

impl<'a, T, C: ComponentMgr> SingleCaseWriteRef<'a, T, C> {
    pub fn new(value: &'a mut SingleCase<T, C>, mgr: usize) -> Self{
        SingleCaseWriteRef{
            mgr: mgr,
            value: value,
        }
    }

    pub fn modify<F: FnOnce(&mut T) -> bool>(&mut self, f: F) {
        if f(&mut self.value) {
            self.handlers.notify(SingleModifyEvent{field: ""}, unsafe{&mut *(self.mgr as *mut C)});
        }
    }
}

//通知处理器
// pub fn notify<T, C: ComponentMgr>(event: Event, handlers: &Vec<Weak<ComponentHandler<T, C>>>, mgr: &mut C) {
//     for it in handlers.iter(){
//         match Weak::upgrade(it) {
//             Some(h) => {
//                 h.handle(&event, mgr);
//             },
//             None => println!("handler has been lost"),
//         }
//     }
// }
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
