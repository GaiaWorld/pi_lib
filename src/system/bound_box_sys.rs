use component::position::{ Position, PositionPoint };
use component::bound_box::{ BoundBoxPoint, BoundBox };
use component::node::{ Node, NodePoint };
use wcs::{ World };

pub struct BoundBoxSys{

}

impl FourFork {
    pub fn new () -> FourFork{
        FourFork{}
    }

    pub fn init (world: &mut World) {
        let world1 = world.clone();
        world.0.borrow_mut().position_component_group.register_moitor(Box::new(move |event: EventType<Position>|{
            world1.four_fork_sys
        })
    }
}