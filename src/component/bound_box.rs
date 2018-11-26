use std::mem::uninitialized;

use wcs::{ID, Thing, Component, EventType, WeakWorld, upgrade_world};

pub struct BoundBox{
    pub min: (f32, f32, f32),
    pub max: (f32, f32, f32),
}

pub mod meta {
    use component::bound_box::BoundBoxMeta;
    lazy_static! {
        pub static ref META: BoundBoxMeta = BoundBoxMeta{id: 11111};
    }
}

impl Component for BoundBox{
    type Meta = BoundBoxMeta;
    type Point = BoundBoxPoint;
    fn meta() -> &'static BoundBoxMeta{
        &meta::META
    }

    //内存未初始化
    fn create_point() -> Self::Point{
        let r = unsafe{ uninitialized() };
        r
    }
}

#[derive(Clone)]
pub struct BoundBoxPoint{
    id: usize,
    world: WeakWorld,
}

impl BoundBoxPoint {
    pub fn set_min_0(&mut self, value: f32) {
        let world = upgrade_world(&self.world);
        let mut borrow_mut = world.borrow_mut();
        borrow_mut.bound_box_component_group.get_mut(self.id()).min.0 = value;
        borrow_mut.bound_box_component_group.notify_moitor(EventType::ModifyFieldIndex(self.clone(), "min", 0));
    }

    pub fn set_min_1(&self, value: f32) {
        let world = upgrade_world(&self.world);
        let mut borrow_mut = world.borrow_mut();
        borrow_mut.bound_box_component_group.get_mut(self.id()).min.1 = value;
        borrow_mut.bound_box_component_group.notify_moitor(EventType::ModifyFieldIndex(self.clone(), "min", 1));
    }

    pub fn set_min_2(&self, value: f32) {
        let world = upgrade_world(&self.world);
        let mut borrow_mut = world.borrow_mut();
        borrow_mut.bound_box_component_group.get_mut(self.id()).min.2 = value;
        borrow_mut.bound_box_component_group.notify_moitor(EventType::ModifyFieldIndex(self.clone(), "min", 2));
    }

    pub fn set_max_0(&mut self, value: f32) {
        let world = upgrade_world(&self.world);
        let mut borrow_mut = world.borrow_mut();
        borrow_mut.bound_box_component_group.get_mut(self.id()).max.0 = value;
        borrow_mut.bound_box_component_group.notify_moitor(EventType::ModifyFieldIndex(self.clone(), "max", 0));
    }

    pub fn set_max_1(&self, value: f32) {
        let world = upgrade_world(&self.world);
        let mut borrow_mut = world.borrow_mut();
        borrow_mut.bound_box_component_group.get_mut(self.id()).max.1 = value;
        borrow_mut.bound_box_component_group.notify_moitor(EventType::ModifyFieldIndex(self.clone(), "max", 1));
    }

    pub fn set_max_2(&self, value: f32) {
        let world = upgrade_world(&self.world);
        let mut borrow_mut = world.borrow_mut();
        borrow_mut.bound_box_component_group.get_mut(self.id()).max.2 = value;
        borrow_mut.bound_box_component_group.notify_moitor(EventType::ModifyFieldIndex(self.clone(), "max", 2));
    }

    pub fn get_min(&self) -> &(f32, f32, f32) {
        unsafe{&*(&upgrade_world(&self.world).borrow().bound_box_component_group.get(self.id()).min as *const (f32, f32, f32) )}
    }

    pub fn get_max(&self) -> &(f32, f32, f32) {
        unsafe{&*(&upgrade_world(&self.world).borrow().bound_box_component_group.get(self.id()).max as *const (f32, f32, f32) )}
    }
}

impl ID for BoundBoxPoint{
    fn id(& self) -> usize{
        self.id
    }
    fn set_id(&mut self, id: usize){
        self.id = id;
    }
}

impl Thing for BoundBoxPoint{
    fn set_world(&mut self, world: WeakWorld){
        self.world = world;
    }
}

pub struct BoundBoxMeta{
    pub id: usize
}

impl ID for BoundBoxMeta{
    fn id(&self) -> usize{
        self.id
    }
    fn set_id(&mut self, id: usize){
        self.id = id
    }
}