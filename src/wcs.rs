use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::clone::Clone;

use slab::{Slab};
use component::position::Position;
use component::bound_box::BoundBox;
use component::node::Node;

pub type WeakWorld = Weak<RefCell<WorldImpl>>;
pub struct World(pub Rc<RefCell<WorldImpl>>);


impl World {
    pub fn new() -> World{
        World(Rc::new(RefCell::new(
            WorldImpl{
                position_component_group: ComponentGroup::new(),
                node_component_group: ComponentGroup::new(),
                bound_box_component_group: ComponentGroup::new(),
                four_fork_sys: FourForkSys::new()
            }
        )))
    }

    pub fn downgrade(&self) -> WeakWorld{
        Rc::downgrade(&self.0)
    }
}


pub struct WorldImpl{
    pub position_component_group: ComponentGroup<Position>,
    pub node_component_group: ComponentGroup<Node>,
    pub bound_box_component_group: ComponentGroup<BoundBox>,
    pub four_fork_sys: FourForkSys
}


pub struct ComponentGroup<T: Component>{
    components: Slab<T>,
    monitors: Vec<Box<Fn(EventType<T>)>>
}

impl<T: Component> ComponentGroup<T>{
    pub fn new() -> Self{
        ComponentGroup{
            components: Slab::new(),
            monitors: Vec::new()
        }
    }

    pub fn alloc(&mut self, world: WeakWorld) -> (<T as Component>::Point, &mut T){
        let (id, value) = self.components.alloc();
        let mut point = T::create_point();
        point.set_world(world);
        point.set_id(id);
        (point, value)
    }

    pub fn insert(&mut self, world: WeakWorld, component: T) -> <T as Component>::Point{
        let id = self.components.insert(component);
        let mut point = T::create_point();
        point.set_world(world);
        point.set_id(id);
        point
    }

    pub fn try_remove(&mut self, id: usize) -> Option<T>{
        if !self.components.contains(id){
            return None;
        }
        Some(self.components.remove(id))
    }

    pub fn remove(&mut self, id: usize) -> T {
        self.components.remove(id)
    }

    pub fn try_get(&mut self, id: usize) -> Option<&T>{
        self.components.get(id)
    }

    //这是一个非安全方法
    pub fn get(&self, id: usize) -> &T{
        unsafe{ self.components.get_unchecked(id) }
    }

    pub fn try_get_mut(&mut self, id: usize) -> Option<&mut T>{
        self.components.get_mut(id)
    }

    //这是一个非安全方法
    pub fn get_mut(&mut self, id: usize) -> &mut T{
        unsafe{ self.components.get_unchecked_mut(id) }
    }

    pub fn register_moitor(&mut self, monitor: Box<Fn(EventType<T>)>) -> usize{
        self.monitors.push(monitor);
        self.monitors.len() - 1
    }

    //moitor的容器是一个Vec, 其移除的性能并不高， 如果需要频繁的移除， 考虑使用slab
    pub fn unregister_moitor(&mut self, index: usize) -> Option<Box<Fn(EventType<T>)>>{
        if index >= self.monitors.len(){
            None
        }else {
            Some(self.monitors.remove(index))
        }
    }

    pub fn notify_moitor(&self, event: EventType<T>){
        for it in self.monitors.iter(){
            it(event.clone());
        }
    }
}

pub trait System {
	/**
	 * 初始化
	 */
	/* tslint:disable:no-empty */
	fn init(w: World);
	/**
	 * 运行
	 */
	fn run();
	/**
	 * 销毁
	 */
	fn destroy();
}

pub struct FourForkSys{
    
}

impl FourForkSys{
    pub fn new() -> FourForkSys {
        FourForkSys{}
    }
}

pub enum EventType<'a, T: Component>  {
    ModifyField(<T as Component>::Point, &'a str),
    ModifyIndex(<T as Component>::Point, usize),
    ModifyFieldIndex(<T as Component>::Point, &'a str, usize),
    Create(<T as Component>::Point),
    Delete(<T as Component>::Point)
}

impl<'a, T: Component> Clone for EventType<'a, T>{
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

pub trait ID{
    fn id(&self) -> usize;
    fn set_id(&mut self, id: usize);
}

// 事物， 包含组件和系统
pub trait Thing{
    fn set_world(&mut self, world: WeakWorld);
}

pub trait Component{
    type Meta;
    type Point: Thing + ID + Clone;
    fn meta() -> &'static Self::Meta;
    fn create_point() -> Self::Point;
}

pub fn upgrade_world(world: &WeakWorld) -> Rc<RefCell<WorldImpl>>{
    match world.upgrade() {
        Some(w) => w,
        None => panic!("world lost!"),
    }
}