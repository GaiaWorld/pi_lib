

#![feature(prelude_import)]
#![no_std]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

#[allow(unused_imports)]
#[prelude_import]
use std::prelude::v1::*;

#[macro_use]
extern crate std;

///一个基本的例子， 定义组件， 实体， 系统， 已经如何实例化World并运行（TODO）
#[allow(unused_imports)]
#[macro_use]
extern crate ecs;
extern crate atom;
extern crate map;
extern crate pointer;
extern crate share;

use atom::Atom;

use ecs::{
    Component, CreateEvent, DeleteEvent, Dispatcher, EntityListener, LendMut, ModifyEvent,
    MultiCaseImpl, MultiCaseListener, Runner, SeqDispatcher, SingleCaseImpl, SingleCaseListener,
    World,
};
use map::vecmap::VecMap;

pub struct Position {
    pub x: f32,
    pub y: f32,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for Position {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Position {
                x: ref __self_0_0,
                y: ref __self_0_1,
            } => {
                let mut debug_trait_builder = f.debug_struct("Position");
                let _ = debug_trait_builder.field("x", &&(*__self_0_0));
                let _ = debug_trait_builder.field("y", &&(*__self_0_1));
                debug_trait_builder.finish()
            }
        }
    }
}

impl Component for Position {
    type Storage = VecMap<Self>;
}

pub struct View {
    pub value: usize,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for View {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            View {
                value: ref __self_0_0,
            } => {
                let mut debug_trait_builder = f.debug_struct("View");
                let _ = debug_trait_builder.field("value", &&(*__self_0_0));
                debug_trait_builder.finish()
            }
        }
    }
}

// Entry
pub struct Node;

pub struct SystemDemo;

impl<'a> Runner<'a> for SystemDemo {
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = ();

    fn setup(&mut self, _read: Self::ReadData, _write: Self::WriteData) {}
    fn run(&mut self, _read: Self::ReadData, _write: Self::WriteData) {
        // println!("run SystemDemo");
    }
    fn dispose(&mut self, _read: Self::ReadData, _write: Self::WriteData) {}
}

impl<'a> MultiCaseListener<'a, Node, Position, CreateEvent> for SystemDemo {
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = ();

    fn listen(&mut self, _event: &CreateEvent, _read: Self::ReadData, _write: Self::WriteData) {
        // println!("listen Position create. id:{}, position: {:?}", event.id, read.get(event.id));
    }
}

impl<'a> MultiCaseListener<'a, Node, Position, ModifyEvent> for SystemDemo {
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = ();

    fn listen(&mut self, _event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData) {
        // println!("listen Position modity. id:{}, position: {:?}", event.id, read.get(event.id));
    }
}

impl<'a> MultiCaseListener<'a, Node, Position, DeleteEvent> for SystemDemo {
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = ();

    fn listen(&mut self, _event: &DeleteEvent, _read: Self::ReadData, _write: Self::WriteData) {
        // println!("listen Position delete. id:{}, position: {:?}", event.id, read.get(event.id));
    }
}

//只有修改事件
impl<'a> SingleCaseListener<'a, View, ModifyEvent> for SystemDemo {
    type ReadData = &'a SingleCaseImpl<View>;
    type WriteData = ();

    fn listen(&mut self, _event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData) {
        // println!("slisten View modify. view: {:?}", &read.value);
    }
}

//只有创建和删除事件
impl<'a> EntityListener<'a, Node, CreateEvent> for SystemDemo {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn listen(&mut self, _event: &CreateEvent, _read: Self::ReadData, _write: Self::WriteData) {
        // println!("elisten Node create. node: {:?}", event.id);
    }
}

impl<'a> EntityListener<'a, Node, DeleteEvent> for SystemDemo {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn listen(&mut self, _event: &DeleteEvent, _read: Self::ReadData, _write: Self::WriteData) {
        // println!("elisten Node delete. node: {:?}", event.id);
    }
}

// create entity, component

// modify component

//modify single

//free entity
pub struct CellSystemDemo {
    owner: ::ecs::cell::StdCell<SystemDemo>,
    run_fn: Option<::ecs::system::RunnerFn>,
    dispose_listener_fn: Option<::ecs::system::DisposeFn>,
}
impl CellSystemDemo {
    pub fn new(sys: SystemDemo) -> Self {
        Self {
            owner: ::ecs::cell::StdCell::new(sys),
            run_fn: None,
            dispose_listener_fn: None,
        }
    }
    fn borrow_mut1(&self) -> &mut SystemDemo {
        unsafe { &mut *(&mut *self.owner.borrow_mut() as *mut SystemDemo) }
    }
}
impl ::ecs::system::System for CellSystemDemo {
    fn get_depends(
        &self,
    ) -> (
        Vec<(std::any::TypeId, std::any::TypeId)>,
        Vec<(std::any::TypeId, std::any::TypeId)>,
    ) {
        let mut read_ids = Vec::new();
        let mut write_ids = Vec::new();
        let r_ids = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            CreateEvent,
        >>::ReadData as ::ecs::system::SystemData>::FetchTarget as ::ecs::TypeIds>::type_ids(
        );
        let w_ids = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            CreateEvent,
        >>::WriteData as ::ecs::system::SystemMutData>::FetchTarget as ::ecs::TypeIds>::type_ids(
        );
        read_ids.extend_from_slice(&r_ids);
        write_ids.extend_from_slice(&w_ids);
        let r_ids = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            DeleteEvent,
        >>::ReadData as ::ecs::system::SystemData>::FetchTarget as ::ecs::TypeIds>::type_ids(
        );
        let w_ids = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            DeleteEvent,
        >>::WriteData as ::ecs::system::SystemMutData>::FetchTarget as ::ecs::TypeIds>::type_ids(
        );
        read_ids.extend_from_slice(&r_ids);
        write_ids.extend_from_slice(&w_ids);
        let r_ids = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            ModifyEvent,
        >>::ReadData as ::ecs::system::SystemData>::FetchTarget as ::ecs::TypeIds>::type_ids(
        );
        let w_ids = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            ModifyEvent,
        >>::WriteData as ::ecs::system::SystemMutData>::FetchTarget as ::ecs::TypeIds>::type_ids(
        );
        read_ids.extend_from_slice(&r_ids);
        write_ids.extend_from_slice(&w_ids);
        let r_ids =
            <<<SystemDemo<> as
              ::ecs::system::SingleCaseListener<'_, View,
                                                ModifyEvent>>::ReadData as
             ::ecs::system::SystemData>::FetchTarget as
                ::ecs::TypeIds>::type_ids();
        let w_ids =
            <<<SystemDemo<> as
              ::ecs::system::SingleCaseListener<'_, View,
                                                ModifyEvent>>::WriteData as
             ::ecs::system::SystemMutData>::FetchTarget as
                ::ecs::TypeIds>::type_ids();
        read_ids.extend_from_slice(&r_ids);
        write_ids.extend_from_slice(&w_ids);
        let r_ids =
            <<<SystemDemo<> as
              ::ecs::system::EntityListener<'_, Node, CreateEvent>>::ReadData
             as ::ecs::system::SystemData>::FetchTarget as
                ::ecs::TypeIds>::type_ids();
        let w_ids =
            <<<SystemDemo<> as
              ::ecs::system::EntityListener<'_, Node, CreateEvent>>::WriteData
             as ::ecs::system::SystemMutData>::FetchTarget as
                ::ecs::TypeIds>::type_ids();
        read_ids.extend_from_slice(&r_ids);
        write_ids.extend_from_slice(&w_ids);
        let r_ids =
            <<<SystemDemo<> as
              ::ecs::system::EntityListener<'_, Node, DeleteEvent>>::ReadData
             as ::ecs::system::SystemData>::FetchTarget as
                ::ecs::TypeIds>::type_ids();
        let w_ids =
            <<<SystemDemo<> as
              ::ecs::system::EntityListener<'_, Node, DeleteEvent>>::WriteData
             as ::ecs::system::SystemMutData>::FetchTarget as
                ::ecs::TypeIds>::type_ids();
        read_ids.extend_from_slice(&r_ids);
        write_ids.extend_from_slice(&w_ids);
        let r_ids =
            <<<SystemDemo<> as Runner>::ReadData as
             ::ecs::system::SystemData>::FetchTarget as
                ::ecs::TypeIds>::type_ids();
        let w_ids =
            <<<SystemDemo<> as Runner>::WriteData as
             ::ecs::system::SystemMutData>::FetchTarget as
                ::ecs::TypeIds>::type_ids();
        read_ids.extend_from_slice(&r_ids);
        write_ids.extend_from_slice(&w_ids);
        (read_ids, write_ids)
    }
    fn setup(
        &mut self,
        me: std::sync::Arc<dyn ecs::system::System>,
        world: &::ecs::world::World,
        name: &Atom,
    ) {
        let me: std::sync::Arc<Self> = match ::ecs::system::System::downcast(me) {
            Ok(r) => r,
            Err(_) => {
                panic!("");
                // ::std::rt::begin_panic("downcast err".to_string(),
                //                        &("src\\lib.rs", 107u32, 1u32))
            }
        };
        let mut listen_arr: Vec<(usize, usize)> = Vec::new();
        let me1 = me.clone();
        let read = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            CreateEvent,
        >>::ReadData as ::ecs::system::SystemData>::FetchTarget as ::ecs::Fetch>::fetch(
            world
        );
        let write = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            CreateEvent,
        >>::WriteData as ::ecs::system::SystemMutData>::FetchTarget as ::ecs::Fetch>::fetch(
            world
        );
        let read_data = ::ecs::Lend::lend1(&read);
        let write_data = ::ecs::LendMut::lend_mut1(&write);
        let f = ::ecs::monitor::FnListener(share::Share::new(move |e| {
            let read_data = ::ecs::Lend::lend2(&read, &read_data);
            let write_data = ::ecs::LendMut::lend_mut2(&write, &write_data);
            <SystemDemo as ecs::MultiCaseListener<'_, Node, Position, CreateEvent>>::listen(
                me1.borrow_mut1(),
                e,
                read_data,
                write_data,
            );
        }));
        let setup_target = world.fetch_multi::<Node, Position>().unwrap();
        ::ecs::monitor::Notify::add_create(&*setup_target, f.clone());
        let ptr: (usize, usize) = unsafe { std::mem::transmute(share::Share::into_raw(f.0)) };
        listen_arr.push(ptr);
        let me1 = me.clone();
        let read = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            DeleteEvent,
        >>::ReadData as ::ecs::system::SystemData>::FetchTarget as ::ecs::Fetch>::fetch(
            world
        );
        let write = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            DeleteEvent,
        >>::WriteData as ::ecs::system::SystemMutData>::FetchTarget as ::ecs::Fetch>::fetch(
            world
        );
        let read_data = ::ecs::Lend::lend1(&read);
        let write_data = ::ecs::LendMut::lend_mut1(&write);
        let f = ::ecs::monitor::FnListener(share::Share::new(move |e| {
            let read_data = ::ecs::Lend::lend2(&read, &read_data);
            let write_data = ::ecs::LendMut::lend_mut2(&write, &write_data);
            <SystemDemo as ecs::MultiCaseListener<'_, Node, Position, DeleteEvent>>::listen(
                me1.borrow_mut1(),
                e,
                read_data,
                write_data,
            );
        }));
        let setup_target = world.fetch_multi::<Node, Position>().unwrap();
        ::ecs::monitor::Notify::add_delete(&*setup_target, f.clone());
        let ptr: (usize, usize) = unsafe { std::mem::transmute(share::Share::into_raw(f.0)) };
        listen_arr.push(ptr);
        let me1 = me.clone();
        let read = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            ModifyEvent,
        >>::ReadData as ::ecs::system::SystemData>::FetchTarget as ::ecs::Fetch>::fetch(
            world
        );
        let write = <<<SystemDemo as ::ecs::system::MultiCaseListener<
            '_,
            Node,
            Position,
            ModifyEvent,
        >>::WriteData as ::ecs::system::SystemMutData>::FetchTarget as ::ecs::Fetch>::fetch(
            world
        );
        let read_data = ::ecs::Lend::lend1(&read);
        let write_data = ::ecs::LendMut::lend_mut1(&write);
        let f = ::ecs::monitor::FnListener(share::Share::new(move |e| {
            let read_data = ::ecs::Lend::lend2(&read, &read_data);
            let write_data = ::ecs::LendMut::lend_mut2(&write, &write_data);
            <SystemDemo as ecs::MultiCaseListener<'_, Node, Position, ModifyEvent>>::listen(
                me1.borrow_mut1(),
                e,
                read_data,
                write_data,
            );
        }));
        let setup_target = world.fetch_multi::<Node, Position>().unwrap();
        ::ecs::monitor::Notify::add_modify(&*setup_target, f.clone());
        let ptr: (usize, usize) = unsafe { std::mem::transmute(share::Share::into_raw(f.0)) };
        listen_arr.push(ptr);
        let me1 = me.clone();
        let read =
            <<<SystemDemo<> as
              ::ecs::system::SingleCaseListener<'_, View,
                                                ModifyEvent>>::ReadData as
             ::ecs::system::SystemData>::FetchTarget as
                ::ecs::Fetch>::fetch(world);
        let write =
            <<<SystemDemo<> as
              ::ecs::system::SingleCaseListener<'_, View,
                                                ModifyEvent>>::WriteData as
             ::ecs::system::SystemMutData>::FetchTarget as
                ::ecs::Fetch>::fetch(world);
        let read_data = ::ecs::Lend::lend1(&read);
        let write_data = ::ecs::LendMut::lend_mut1(&write);
        let f = ::ecs::monitor::FnListener(share::Share::new(move |e| {
            let read_data = ::ecs::Lend::lend2(&read, &read_data);
            let write_data = ::ecs::LendMut::lend_mut2(&write, &write_data);
            <SystemDemo as ecs::SingleCaseListener<'_, View, ModifyEvent>>::listen(
                me1.borrow_mut1(),
                e,
                read_data,
                write_data,
            );
        }));
        let setup_target = world.fetch_single::<View>().unwrap();
        ::ecs::monitor::Notify::add_modify(&*setup_target, f.clone());
        let ptr: (usize, usize) = unsafe { std::mem::transmute(share::Share::into_raw(f.0)) };
        listen_arr.push(ptr);
        let me1 = me.clone();
        let read =
            <<<SystemDemo<> as
              ::ecs::system::EntityListener<'_, Node, CreateEvent>>::ReadData
             as ::ecs::system::SystemData>::FetchTarget as
                ::ecs::Fetch>::fetch(world);
        let write =
            <<<SystemDemo<> as
              ::ecs::system::EntityListener<'_, Node, CreateEvent>>::WriteData
             as ::ecs::system::SystemMutData>::FetchTarget as
                ::ecs::Fetch>::fetch(world);
        let read_data = ::ecs::Lend::lend1(&read);
        let write_data = ::ecs::LendMut::lend_mut1(&write);
        let f = ::ecs::monitor::FnListener(share::Share::new(move |e| {
            let read_data = ::ecs::Lend::lend2(&read, &read_data);
            let write_data = ::ecs::LendMut::lend_mut2(&write, &write_data);
            <SystemDemo as ecs::EntityListener<'_, Node, CreateEvent>>::listen(
                me1.borrow_mut1(),
                e,
                read_data,
                write_data,
            );
        }));
        let setup_target = world.fetch_entity::<Node>().unwrap();
        ::ecs::monitor::Notify::add_create(&*setup_target, f.clone());
        let ptr: (usize, usize) = unsafe { std::mem::transmute(share::Share::into_raw(f.0)) };
        listen_arr.push(ptr);
        let me1 = me.clone();
        let read =
            <<<SystemDemo<> as
              ::ecs::system::EntityListener<'_, Node, DeleteEvent>>::ReadData
             as ::ecs::system::SystemData>::FetchTarget as
                ::ecs::Fetch>::fetch(world);
        let write =
            <<<SystemDemo<> as
              ::ecs::system::EntityListener<'_, Node, DeleteEvent>>::WriteData
             as ::ecs::system::SystemMutData>::FetchTarget as
                ::ecs::Fetch>::fetch(world);
        let read_data = ::ecs::Lend::lend1(&read);
        let write_data = ::ecs::LendMut::lend_mut1(&write);
        let f = ::ecs::monitor::FnListener(share::Share::new(move |e| {
            let read_data = ::ecs::Lend::lend2(&read, &read_data);
            let write_data = ::ecs::LendMut::lend_mut2(&write, &write_data);
            <SystemDemo as ecs::EntityListener<'_, Node, DeleteEvent>>::listen(
                me1.borrow_mut1(),
                e,
                read_data,
                write_data,
            );
        }));
        let setup_target = world.fetch_entity::<Node>().unwrap();
        ::ecs::monitor::Notify::add_delete(&*setup_target, f.clone());
        let ptr: (usize, usize) = unsafe { std::mem::transmute(share::Share::into_raw(f.0)) };
        listen_arr.push(ptr);
        let read =
            <<<SystemDemo<> as ::ecs::system::Runner>::ReadData as
             ::ecs::system::SystemData>::FetchTarget as
                ::ecs::Fetch>::fetch(world);
        let write =
            <<<SystemDemo<> as ::ecs::system::Runner>::WriteData as
             ::ecs::system::SystemMutData>::FetchTarget as
                ::ecs::Fetch>::fetch(world);
        let read_data = ::ecs::Lend::lend1(&read);
        let write_data = ::ecs::LendMut::lend_mut1(&write);
        {
            let read_data = ::ecs::Lend::lend2(&read, &read_data);
            let write_data = ::ecs::LendMut::lend_mut2(&write, &write_data);
            self.borrow_mut1().setup(read_data, write_data);
        }
        let runtime = world.runtime.clone();
        let runtime_ref = unsafe {
            &mut *(runtime.as_ref() as *const Vec<::ecs::RunTime> as *mut Vec<::ecs::RunTime>)
        };
        let runtime_index = runtime_ref.len();
        runtime_ref.push(::ecs::RunTime {
            sys_name: name.clone(),
            cost_time: std::time::Duration::from_millis(0),
        });
        self.run_fn = Some(::ecs::monitor::FnListener(share::Share::new(
            move |_e: &()| {
                let runtime_ref = unsafe {
                    &mut *(runtime.as_ref() as *const Vec<::ecs::RunTime>
                        as *mut Vec<::ecs::RunTime>)
                };
                let time = std::time::Instant::now();
                let read_data = ::ecs::Lend::lend2(&read, &read_data);
                let write_data = ::ecs::LendMut::lend_mut2(&write, &write_data);
                me.borrow_mut1().run(read_data, write_data);
                runtime_ref[runtime_index].cost_time = std::time::Instant::now() - time;
            },
        )));
        self.dispose_listener_fn = Some(::ecs::monitor::FnListener(share::Share::new(
            move |world: &::ecs::world::World| {
                let setup_target = world.fetch_multi::<Node, Position>().unwrap();
                let r: Box<dyn Fn(&CreateEvent)> =
                    unsafe { std::mem::transmute((&listen_arr[0]).clone()) };
                let r: ::ecs::monitor::FnListener<CreateEvent> =
                    ::ecs::monitor::FnListener(unsafe { share::Share::from_raw(Box::into_raw(r)) });
                ::ecs::monitor::Notify::remove_create(&*setup_target, &r);
                let setup_target = world.fetch_multi::<Node, Position>().unwrap();
                let r: Box<dyn Fn(&DeleteEvent)> =
                    unsafe { std::mem::transmute((&listen_arr[0 + 1]).clone()) };
                let r: ::ecs::monitor::FnListener<DeleteEvent> =
                    ::ecs::monitor::FnListener(unsafe { share::Share::from_raw(Box::into_raw(r)) });
                ::ecs::monitor::Notify::remove_delete(&*setup_target, &r);
                let setup_target = world.fetch_multi::<Node, Position>().unwrap();
                let r: Box<dyn Fn(&ModifyEvent)> =
                    unsafe { std::mem::transmute((&listen_arr[0 + 1 + 1]).clone()) };
                let r: ::ecs::monitor::FnListener<ModifyEvent> =
                    ::ecs::monitor::FnListener(unsafe { share::Share::from_raw(Box::into_raw(r)) });
                ::ecs::monitor::Notify::remove_modify(&*setup_target, &r);
                let setup_target = world.fetch_single::<View>().unwrap();
                let r: Box<dyn Fn(&ModifyEvent)> =
                    unsafe { std::mem::transmute((&listen_arr[0 + 1 + 1 + 1]).clone()) };
                let r: ::ecs::monitor::FnListener<ModifyEvent> =
                    ::ecs::monitor::FnListener(unsafe { share::Share::from_raw(Box::into_raw(r)) });
                ::ecs::monitor::Notify::remove_modify(&*setup_target, &r);
                let setup_target = world.fetch_entity::<Node>().unwrap();
                let r: Box<dyn Fn(&CreateEvent)> =
                    unsafe { std::mem::transmute((&listen_arr[0 + 1 + 1 + 1 + 1]).clone()) };
                let r: ::ecs::monitor::FnListener<CreateEvent> =
                    ::ecs::monitor::FnListener(unsafe { share::Share::from_raw(Box::into_raw(r)) });
                ::ecs::monitor::Notify::remove_create(&*setup_target, &r);
                let setup_target = world.fetch_entity::<Node>().unwrap();
                let r: Box<dyn Fn(&DeleteEvent)> =
                    unsafe { std::mem::transmute((&listen_arr[0 + 1 + 1 + 1 + 1 + 1]).clone()) };
                let r: ::ecs::monitor::FnListener<DeleteEvent> =
                    ::ecs::monitor::FnListener(unsafe { share::Share::from_raw(Box::into_raw(r)) });
                ::ecs::monitor::Notify::remove_delete(&*setup_target, &r);
            },
        )));
    }
    fn dispose(&self, world: &::ecs::world::World) {
        match &self.dispose_listener_fn {
            Some(f) => (f.0)(world),
            None => (),
        };
        let read =
            <<<SystemDemo<> as ::ecs::system::Runner>::ReadData as
             ::ecs::system::SystemData>::FetchTarget as
                ::ecs::Fetch>::fetch(world);
        let write =
            <<<SystemDemo<> as ::ecs::system::Runner>::WriteData as
             ::ecs::system::SystemMutData>::FetchTarget as
                ::ecs::Fetch>::fetch(world);
        let read_data = ::ecs::Lend::lend(&read);
        let write_data = ::ecs::LendMut::lend_mut(&write);
        self.borrow_mut1().dispose(read_data, write_data);
    }
    fn fetch_run(&self) -> Option<::ecs::system::RunnerFn> {
        self.run_fn.clone()
    }
}
fn main() {
    let mut world = World::default();
    let system_demo = CellSystemDemo::new(SystemDemo);
    world.register_entity::<Node>();
    world.register_multi::<Node, Position>();
    world.register_single::<View>(View { value: 6 });
    world.register_system(Atom::from("system_demo"), system_demo);
    let e = world.create_entity::<Node>();
    let position = Position { x: 5.0, y: 5.0 };
    let positions = world.fetch_multi::<Node, Position>().unwrap();
    let positions = LendMut::lend_mut(&positions);
    positions.insert(e, position);
    let write = unsafe { positions.get_unchecked_write(e) };
    write.value.x = 10.0;
    write.notify.modify_event(write.id, "x", 0);
    let view = world.fetch_single::<View>().unwrap();
    let view = LendMut::lend_mut(&view);
    let write = view.get_write();
    write.value.value = 10;
    write.notify.modify_event(write.id, "value", 0);
    let mut dispatch = SeqDispatcher::default();
    dispatch.build("system_demo".to_string(), &world);
    dispatch.run();
    world.free_entity::<Node>(e);
}
