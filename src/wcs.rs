use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::clone::Clone;

use slab::{Slab};

pub type WeakWorld<C, E> = Weak<RefCell<WorldImpl<C, E>>>;
pub struct World<C: ComponentMgr, E>(pub Rc<RefCell<WorldImpl<C, E>>>);

impl<C: ComponentMgr, E> World<C, E> {
    pub fn new() -> World<C, E>{
        World(Rc::new(RefCell::new(
            WorldImpl{
                component_mgr : C::new(),
                runners: Vec::new(),
            }
        )))
    }

    pub fn downgrade(&self) -> WeakWorld<C, E>{
        Rc::downgrade(&self.0)
    }

    pub fn set_runners(&self, list: Vec<Rc<Runner<E>>>){
        self.0.borrow_mut().runners = list;
    }

    pub fn run(&self, e: E){
        let mut borrow_mut = self.0.borrow_mut();
        for s in borrow_mut.runners.iter_mut(){
            s.run(&e);
        }
    }
}


pub struct WorldImpl<C: ComponentMgr, E>{
    component_mgr : C,
    runners: Vec<Rc<Runner<E>>>,
}

pub fn upgrade_world<C: ComponentMgr, E>(world: &WeakWorld<C, E>) -> Rc<RefCell<WorldImpl<C, E>>>{
    match world.upgrade() {
        Some(w) => w,
        None => panic!("world lost!"),
    }
}

pub trait ComponentMgr: 'static + Sized{
    fn new() -> Self;
}

pub trait System<E>{
    fn run(&mut self, e: &E);
    fn init<C: ComponentMgr>(self, world: World<C, E>) -> Rc<RefCell<Self>>;
}

pub trait ID{
    fn id(&self) -> usize;
    fn set_id(&mut self, id: usize);
}

pub trait Runner<E>{
    fn run(&self, e: &E);
}

impl<E, T: System<E>> Runner<E> for RefCell<T>{
    fn run(&self, e: &E){
        self.borrow_mut().run(e);
    }
}


// pub struct ComponentGroup<T: Component>{
//     components: Slab<T>,
//     monitors: Vec<Box<Fn(EventType<T>)>>
// }

// impl<T: Component> ComponentGroup<T>{
//     pub fn new() -> Self{
//         ComponentGroup{
//             components: Slab::new(),
//             monitors: Vec::new()
//         }
//     }

//     pub fn alloc(&mut self, world: WeakWorld<$(ComponentMgr), $(SystemMgr)>) -> (<T as Component>::Point, &mut T){
//         let (id, value) = self.components.alloc();
//         let mut point = T::create_point();
//         point.set_world(world);
//         point.set_id(id);
//         (point, value)
//     }

//     pub fn insert(&mut self, world: WeakWorld<$(ComponentMgr), $(SystemMgr)>, component: T) -> <T as Component>::Point{
//         let id = self.components.insert(component);
//         let mut point = T::create_point();
//         point.set_world(world);
//         point.set_id(id);
//         point
//     }

//     pub fn try_remove(&mut self, id: usize) -> Option<T>{
//         if !self.components.contains(id){
//             return None;
//         }
//         Some(self.components.remove(id))
//     }

//     pub fn remove(&mut self, id: usize) -> T {
//         self.components.remove(id)
//     }

//     pub fn try_get(&mut self, id: usize) -> Option<&T>{
//         self.components.get(id)
//     }

//     //这是一个非安全方法
//     pub fn get(&self, id: usize) -> &T{
//         unsafe{ self.components.get_unchecked(id) }
//     }

//     pub fn try_get_mut(&mut self, id: usize) -> Option<&mut T>{
//         self.components.get_mut(id)
//     }

//     //这是一个非安全方法
//     pub fn get_mut(&mut self, id: usize) -> &mut T{
//         unsafe{ self.components.get_unchecked_mut(id) }
//     }

//     pub fn register_moitor(&mut self, monitor: Box<Fn(EventType<T>)>) -> usize{
//         self.monitors.push(monitor);
//         self.monitors.len() - 1
//     }

//     //moitor的容器是一个Vec, 其移除的性能并不高， 如果需要频繁的移除， 考虑使用slab
//     pub fn unregister_moitor(&mut self, index: usize) -> Option<Box<Fn(EventType<T>)>>{
//         if index >= self.monitors.len(){
//             None
//         }else {
//             Some(self.monitors.remove(index))
//         }
//     }

//     pub fn notify_moitor(&self, event: EventType<T>){
//         for it in self.monitors.iter(){
//             it(event.clone());
//         }
//     }
// }

// pub enum EventType<'a, T: Component>  {
//     ModifyField(<T as Component>::Point, &'a str),
//     ModifyIndex(<T as Component>::Point, usize),
//     ModifyFieldIndex(<T as Component>::Point, &'a str, usize),
//     Create(<T as Component>::Point),
//     Delete(<T as Component>::Point)
// }

// impl<'a, T: Component> Clone for EventType<'a, T>{
//     fn clone (&self) -> Self {
//         match self {
//             EventType::ModifyField(t, ref s) => EventType::ModifyField(t.clone(), s),
//             EventType::ModifyIndex(t, ref u) => EventType::ModifyIndex(t.clone(), u.clone()),
//             EventType::ModifyFieldIndex(t, ref s, u) => EventType::ModifyFieldIndex(t.clone(), s, u.clone()) ,
//             EventType::Create(t) => EventType::Create(t.clone()),
//             EventType::Delete(t) => EventType::Delete(t.clone())
//         }
//     }
// }

// 事物， 包含组件和系统
// pub trait Thing{
//     fn set_world(&mut self, world: WeakWorld<$(ComponentMgr), $(E)>);
// }

// pub trait Component{
//     type Meta;
//     type Point: Thing + ID + Clone;
//     fn meta() -> &'static Self::Meta;
//     fn create_point() -> Self::Point;
// }