///组件， 实体， 系统经过宏展开后的最终代码呈现

extern crate ecs;
extern crate map;
extern crate listener;
extern crate pointer;
extern crate any;

use std::any::TypeId;
use std::sync::Arc;

use pointer::cell::{TrustCell};

use ecs::component::{ Component, MultiCaseImpl};
use ecs::system::{Runner, System, RunnerFn, SystemData, SystemMutData, MultiCaseListener, DisposeFn};
use ecs::monitor::{Notify, CreateEvent};
use listener::{FnListener};
use ecs::world:: { World, Fetch, Borrow, BorrowMut, TypeIds};
use map::vecmap::VecMap;

struct Position{}

impl Component for Position{
    type Strorage = VecMap<Self>;
}


// Entry
struct Node;


struct SystemDemo;

impl<'a> Runner<'a> for SystemDemo{
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn setup(&mut self, read: Self::ReadData, write: Self::WriteData) {}
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {}
    fn dispose(&mut self, read: Self::ReadData, write: Self::WriteData) {}
}

impl<'a> MultiCaseListener<'a, Node, Position, CreateEvent> for SystemDemo {
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData) {}
}

struct CellSystemDemo{
    owner: TrustCell<SystemDemo>,
    run_fn: Option<RunnerFn>,
    dispose_listener_fn: Option<DisposeFn>,
}

impl System for CellSystemDemo {
    fn setup(&mut self, me: Arc<System>, world: &World){
        let me: Arc<Self> = match me.downcast() {
            Ok(r) => r,
            Err(_) => panic!("downcast err".to_string()),
        };
        //listen
        let (f1,) = ({
            let me = me.clone();
            let read = <<SystemDemo as MultiCaseListener<'_, Node, Position, CreateEvent>>::ReadData as SystemData>::FetchTarget::fetch(world);
            let write = <<SystemDemo as MultiCaseListener<'_, Node, Position, CreateEvent>>::WriteData as SystemMutData>::FetchTarget::fetch(world);
            let f = FnListener(Arc::new( move |e: &CreateEvent| {
                let read_data = read.borrow();
                let write_data = write.borrow_mut();
                me.owner.borrow_mut().listen(e, read_data, write_data);
            }));
            let setup_target = world.fetch_multi::<Node, Position>().unwrap();
            Notify::add_create(&*setup_target, f.clone());
            f
        },);

        //run
        let read = <<SystemDemo as Runner>::ReadData as SystemData>::FetchTarget::fetch(world);
        let write = <<SystemDemo as Runner>::WriteData as SystemMutData>::FetchTarget::fetch(world);
        {
            let read_data = read.borrow();
            let write_data = write.borrow_mut();
            self.owner.borrow_mut().setup(read_data, write_data);
        }
        self.run_fn = Some(FnListener(Arc::new( move |e: &()| {
            let read_data = read.borrow();
            let write_data = write.borrow_mut();
            me.owner.borrow_mut().run(read_data, write_data);
        })));
        

        //dispose
        self.dispose_listener_fn =  Some(FnListener(Arc::new(move |world: &World| {
            let setup_target = world.fetch_multi::<Node, Position>().unwrap();
            Notify::remove_create(&*setup_target, &f1);
        })));
    }
    fn dispose(&mut self, world: &World){
        match &self.dispose_listener_fn {
            Some(f) => f.0(world),
            None => (),
        };

        // runner dispose
        let read = <<SystemDemo as MultiCaseListener<'_, Node, Position, CreateEvent>>::ReadData as SystemData>::FetchTarget::fetch(world);
        let write = <<SystemDemo as MultiCaseListener<'_, Node, Position, CreateEvent>>::WriteData as SystemMutData>::FetchTarget::fetch(world);
        let read_data = read.borrow();
        let write_data = write.borrow_mut();
        self.owner.borrow_mut().dispose(read_data, write_data);

        self.dispose_listener_fn = None;
        self.run_fn = None;

    }
    fn get_depends(&self) -> (Vec<(TypeId, TypeId)>, Vec<(TypeId, TypeId)>){
        let mut read_ids = Vec::new();
        let mut write_ids = Vec::new();

        let r_ids = <<SystemDemo as Runner>::ReadData as SystemData>::FetchTarget::type_ids();
        let w_ids = <<SystemDemo as Runner>::WriteData as SystemMutData>::FetchTarget::type_ids();
        read_ids.extend_from_slice(&r_ids);
        write_ids.extend_from_slice(&w_ids);

        let r_ids = <<SystemDemo as MultiCaseListener<'_, Node, Position, CreateEvent>>::ReadData as SystemData>::FetchTarget::type_ids();
        let w_ids = <<SystemDemo as MultiCaseListener<'_, Node, Position, CreateEvent>>::WriteData as SystemMutData>::FetchTarget::type_ids();
        read_ids.extend_from_slice(&r_ids);
        write_ids.extend_from_slice(&w_ids);

        (read_ids, write_ids)
    }
    fn fetch_run(&self, world: &World) -> Option<RunnerFn>{
        self.run_fn.clone()
    }
}
