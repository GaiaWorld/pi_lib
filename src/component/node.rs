use std::mem::uninitialized;

use wcs::{ID, Thing, Component, EventType, upgrade_world, WeakWorld};
use component::position::{ Position, PositionPoint };
use component::bound_box::{BoundBoxPoint, BoundBox};

pub struct Node{
    pub position: PositionPoint,
    pub bound_box: BoundBoxPoint,
}

pub mod meta {
    use component::node::NodeMeta;
    lazy_static! {
        pub static ref META: NodeMeta = NodeMeta{id: 11111};
    }
}

impl Component for Node{
    type Meta = NodeMeta;
    type Point = NodePoint;
    fn meta() -> &'static NodeMeta{
        &meta::META
    }

    //内存未初始化
    fn create_point() -> Self::Point{
        let r = unsafe{ uninitialized() };
        r
    }
}

#[derive(Clone)]
pub struct NodePoint{
    id: usize,
    world: WeakWorld,
}

impl NodePoint {
    pub fn set_position(&mut self, value: Position) {
        let world = upgrade_world(&self.world);
        let mut borrow_mut = world.borrow_mut();
        let point = borrow_mut.position_component_group.insert(self.world.clone(), value);
        borrow_mut.node_component_group.get_mut(self.id()).position = point;
        borrow_mut.node_component_group.notify_moitor(EventType::ModifyField(self.clone(), "position"));
    }

    pub fn set_box(&mut self, value: BoundBox) {
        let world = upgrade_world(&self.world);
        let mut borrow_mut = world.borrow_mut();
        let point = borrow_mut.bound_box_component_group.insert(self.world.clone(), value);
        borrow_mut.node_component_group.get_mut(self.id()).bound_box = point;
        borrow_mut.node_component_group.notify_moitor(EventType::ModifyField(self.clone(), "box"));
    }

    pub fn get_position(&self) -> &PositionPoint {
        let world = upgrade_world(&self.world);
        let borrow = world.borrow();
        unsafe{&*(&borrow.node_component_group.get(self.id()).position as *const PositionPoint)}
    }

    pub fn get_bound_box(&self) -> &BoundBoxPoint {
        let world = upgrade_world(&self.world);
        let borrow = world.borrow();
        unsafe{&*(&borrow.node_component_group.get(self.id()).bound_box as *const BoundBoxPoint)}
    }
}

impl ID for NodePoint{
    fn id(& self) -> usize{
        self.id
    }
    fn set_id(&mut self, id: usize){
        self.id = id;
    }
}

impl Thing for NodePoint{
    fn set_world(&mut self, world: WeakWorld){
        self.world = world;
    }
}

pub struct NodeMeta{
    pub id: usize
}

impl ID for NodeMeta{
    fn id(&self) -> usize{
        self.id
    }
    fn set_id(&mut self, id: usize){
        self.id = id
    }
}