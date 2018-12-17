extern crate slab;

use std::rc::Rc;
use std::cell::RefCell;

impl<C: ComponentMgr, E> World<C, E> {
    pub fn new() -> World<C, E>{
        World{
            component_mgr : C::new(),
            systems: Vec::new(),
        }
    }

    pub fn set_systems(&mut self, list: Vec<Rc<System<E, C>>>){
        self.systems = list;
    }

    pub fn run(&mut self, e: E){
        let mut c_mgr = self.component_mgr.borrow_mut();
        for runner in self.systems.iter(){
            runner.run(&e, &mut *c_mgr);
        }
    }
}


pub struct World<C: ComponentMgr, E>{
    pub component_mgr : Rc<RefCell<C>>,
    systems: Vec<Rc<System<E, C>>>,
}

pub trait ComponentMgr: 'static + Sized{
    fn new() -> Rc<RefCell<Self>>;
}

pub trait System<E, C: ComponentMgr>{
    fn run(&self, e: &E, w: &mut C);
    fn init(w: &mut C) -> Rc<Self> where Self: Sized;
}

pub trait ID{
    fn id(&self) -> usize;
    fn set_id(&mut self, id: usize);
}
