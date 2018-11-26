

use std::mem::uninitialized;
use wcs::{ID, Thing, Component, EventType, WeakWorld, upgrade_world};

pub mod meta {
    use component::position::PositionMeta;
    lazy_static! {
        pub static ref META: PositionMeta = PositionMeta{id: 11111};
    }
}

pub struct Position{
    pub x: f32,
    pub y:f32,
    pub z:f32
}

impl Component for Position{
    type Meta = PositionMeta;
    type Point = PositionPoint;
    fn meta() -> &'static PositionMeta{
        &meta::META
    }

    //内存未初始化
    fn create_point() -> Self::Point{
        let r = unsafe{ uninitialized() };
        r
    }
}

#[derive(Clone)]
pub struct PositionPoint{
    id: usize,
    world: WeakWorld,
}

impl PositionPoint {
    pub fn set_x(&mut self, value: f32) {
        let world = upgrade_world(&self.world);
        let mut borrow_mut = world.borrow_mut();
        borrow_mut.position_component_group.get_mut(self.id()).x = value;
        borrow_mut.position_component_group.notify_moitor(EventType::ModifyField(self.clone(), "x"));
    }

    pub fn set_y(&mut self, value: f32) {
        let world = upgrade_world(&self.world);
        let mut borrow_mut = world.borrow_mut();
        borrow_mut.position_component_group.get_mut(self.id()).y = value;
        borrow_mut.position_component_group.notify_moitor(EventType::ModifyField(self.clone(), "y"));
    }

    pub fn set_z(&mut self, value: f32) {
        let world = upgrade_world(&self.world);
        let mut borrow_mut = world.borrow_mut();
        borrow_mut.position_component_group.get_mut(self.id()).z = value;
        borrow_mut.position_component_group.notify_moitor(EventType::ModifyField(self.clone(), "z"));
    }

    pub fn get_x(&self) -> &f32 {
        unsafe{&*(&upgrade_world(&self.world).borrow().position_component_group.get(self.id()).z as *const f32)}
    }

    pub fn get_y(&self) -> &f32 {
        unsafe{&*(&upgrade_world(&self.world).borrow().position_component_group.get(self.id()).x as *const f32)}
    }

    pub fn get_z(&self) -> &f32 {
        unsafe{&*(&upgrade_world(&self.world).borrow().position_component_group.get(self.id()).z as *const f32)}
    }

}

impl ID for PositionPoint{
    fn id(& self) -> usize{
        self.id
    }
    fn set_id(&mut self, id: usize){
        self.id = id;
    }
}

impl Thing for PositionPoint{
    fn set_world(&mut self, world: WeakWorld){
        self.world = world;
    }
}

pub struct PositionMeta{
    pub id: usize
}

impl ID for PositionMeta{
    fn id(&self) -> usize{
        self.id
    }
    fn set_id(&mut self, id: usize){
        self.id = id
    }
}