use component::position::{ Position, PositionPoint };
use component::bound_box::{ BoundBoxPoint, BoundBox };
use component::node::{ Node, NodePoint };
use wcs::{ World, System, Share, ComponentMgr};
use std::rc::{Rc, Weak};
use std::cell::RefCell;

type ShareFourFork = Rc<RefCell<FourFork>>;

pub struct FourFork<E>{
}

impl FourFork<E> {
    pub fn new () ->  FourFork{
        FourFork{}
    }

    pub fn init<C: ComponentMgr> (self, world: &mut World<C, ComponentMgr>) -> ShareFourFork {
        let share = Rc::new(RefCell::new(self));
        let world1 = world.clone();
        let share1 = share.clone();
        let borrow_mut = world.0.borrow_mut();
        borrow_mut.position_component_group.register_moitor(Box::new(move |event: EventType<Position>|{
            //world1.four_fork_sys
        });
        share
    }
}

impl FourFork for Runner<E>{
    fn run(&mut self, e: &E) {

    }
}

impl FourFork for System<E>{
}

