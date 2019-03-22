extern crate slab;

use std::rc::Rc;

impl<C: ComponentMgr, E> World<C, E> {
    pub fn new(mgr: C) -> World<C, E>{
        World{
            component_mgr : mgr,
            systems: Vec::new(),
        }
    }

    pub fn set_systems(&mut self, list: Vec<Rc<System<E, C>>>){
        self.systems = list;
    }

    pub fn run(&mut self, e: E){
        let mut c_mgr = &mut self.component_mgr;
        for runner in self.systems.iter(){
            runner.run(&e, &mut c_mgr);
        }
    }
}

pub struct World<C: ComponentMgr, E>{
    pub component_mgr : C,
    systems: Vec<Rc<System<E, C>>>,
}

impl<C: ComponentMgr + Default, E> Default for World<C, E> {
    fn default() -> Self {
        World{
            component_mgr: C::default(),
            systems: Vec::new(),
        }
    }
}

pub trait ComponentMgr: 'static + Sized{}

pub trait System<E, C: ComponentMgr>{
    fn run(&self, e: &E, w: &mut C);
}

// pub trait ID{
//     fn id(&self) -> usize;
//     fn set_id(&mut self, id: usize);
// }
