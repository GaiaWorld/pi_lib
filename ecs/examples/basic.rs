#![feature(proc_macro_hygiene)]

///一个基本的例子， 定义组件， 实体， 系统， 已经如何实例化World并运行（TODO）

#[macro_use]
extern crate ecs;
extern crate map;
extern crate pointer;
extern crate atom;
extern crate share;

use atom::Atom;

use ecs::{Component, MultiCaseImpl, SingleCaseImpl, Runner, MultiCaseListener, SingleCaseListener, EntityListener, CreateEvent, ModifyEvent, DeleteEvent, LendMut, World, SeqDispatcher, Dispatcher};
use map::vecmap::VecMap;

#[derive(Debug)]
pub struct Position{
    pub x: f32,
    pub y: f32,
}

impl Component for Position{
    type Storage = VecMap<Self>;
}

#[derive(Debug)]
pub struct View{
    pub value: usize,
}

// Entry
pub struct Node;


pub struct SystemDemo;

impl<'a> Runner<'a> for SystemDemo{
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = ();

    fn setup(&mut self, _read: Self::ReadData, _write: Self::WriteData) {}
    fn run(&mut self, _read: Self::ReadData, _write: Self::WriteData) {
        println!("run SystemDemo");
    }
    fn dispose(&mut self, _read: Self::ReadData, _write: Self::WriteData) {}
}

impl<'a> MultiCaseListener<'a, Node, Position, CreateEvent> for SystemDemo {
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = ();

    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, _write: Self::WriteData) {
        println!("listen Position create. id:{}, position: {:?}", event.id, read.get(event.id));
    }
}

impl<'a> MultiCaseListener<'a, Node, Position, ModifyEvent> for SystemDemo {
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = ();

    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData) {
        println!("listen Position modity. id:{}, position: {:?}", event.id, read.get(event.id));
    }
}

impl<'a> MultiCaseListener<'a, Node, Position, DeleteEvent> for SystemDemo {
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = ();

    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, _write: Self::WriteData) {
        println!("listen Position delete. id:{}, position: {:?}", event.id, read.get(event.id));
    }
}

//只有修改事件
impl<'a> SingleCaseListener<'a, View, ModifyEvent> for SystemDemo {
    type ReadData = &'a SingleCaseImpl<View>;
    type WriteData = ();

    fn listen(&mut self, _event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData) {
        println!("slisten View modify. view: {:?}", &read.value);
    }
}

//只有创建和删除事件
impl<'a> EntityListener<'a, Node, CreateEvent> for SystemDemo {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, _write: Self::WriteData) {
        println!("elisten Node create. node: {:?}", event.id);
    }
}

impl<'a> EntityListener<'a, Node, DeleteEvent> for SystemDemo {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, _write: Self::WriteData) {
        println!("elisten Node delete. node: {:?}", event.id);
    }
}


impl_system!{
    SystemDemo,
    true,
    {
        MultiCaseListener<Node, Position, CreateEvent>
        MultiCaseListener<Node, Position, DeleteEvent>
        MultiCaseListener<Node, Position, ModifyEvent>
        SingleCaseListener<View, ModifyEvent>
        EntityListener<Node, CreateEvent>
        EntityListener<Node, DeleteEvent>
    }
}

fn main() { 
    let mut world = World::default();
    let system_demo = CellSystemDemo::new(SystemDemo);

    world.register_entity::<Node>();
    world.register_multi::<Node, Position>();
    world.register_single::<View>(View{value: 6});

    world.register_system(Atom::from("system_demo"), system_demo);

    // create entity, component
    let e = world.create_entity::<Node>();
    let position = Position {x: 5.0, y: 5.0};
    let positions = world.fetch_multi::<Node, Position>().unwrap();
    let positions = LendMut::lend_mut(&positions);
    positions.insert(e, position);

    // modify component
    let write = unsafe { positions.get_unchecked_write(e) };
    write.value.x = 10.0;
    write.notify.modify_event(write.id, "x", 0);

    //modify single
    let view = world.fetch_single::<View>().unwrap();
    let view = LendMut::lend_mut(&view);
    let write = view.get_write();
    write.value.value = 10;
    write.notify.modify_event(write.id, "value", 0);


    let mut dispatch = SeqDispatcher::default();
    dispatch.build("system_demo".to_string(), &world);
    dispatch.run();

    //free entity
    world.free_entity::<Node>(e);
}