#![feature(proc_macro_hygiene)]

///一个基本的例子， 定义组件， 实体， 系统， 已经如何实例化World并运行（TODO）

#[macro_use]
extern crate ecs;
extern crate map;
extern crate pointer;

use ecs::component::{ Component, MultiCaseImpl};
use ecs::single::SingleCaseImpl;
use ecs::system::{Runner, MultiCaseListener, SingleCaseListener, EntityListener};
use ecs::monitor::{CreateEvent, ModifyEvent, DeleteEvent};
use map::vecmap::VecMap;

#[derive(Debug)]
pub struct Position{
    pub x: f32,
    pub y: f32,
}

impl Component for Position{
    type Storage = VecMap<Self>;
}


pub struct View{
    pub value: usize,
}

// Entry
pub struct Node;


pub struct SystemDemo;

impl<'a> Runner<'a> for SystemDemo{
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn setup(&mut self, _read: Self::ReadData, _write: Self::WriteData) {}
    fn run(&mut self, _read: Self::ReadData, _write: Self::WriteData) {}
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
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn listen(&mut self, _event: &DeleteEvent, _read: Self::ReadData, _write: Self::WriteData) {

    }
}

impl<'a> SingleCaseListener<'a, View, ModifyEvent> for SystemDemo {
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn slisten(&mut self, _event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData) {

    }
}

impl<'a> EntityListener<'a, Node, ModifyEvent> for SystemDemo {
    type ReadData = &'a SingleCaseImpl<View>;
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn elisten(&mut self, _event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData) {

    }
}

impl<'a> EntityListener<'a, Node, CreateEvent> for SystemDemo {
    type ReadData = &'a SingleCaseImpl<View>;
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn elisten(&mut self, _event: &CreateEvent, _read: Self::ReadData, _write: Self::WriteData) {

    }
}

impl<'a> EntityListener<'a, Node, DeleteEvent> for SystemDemo {
    type ReadData = &'a SingleCaseImpl<View>;
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn elisten(&mut self, _event: &DeleteEvent, _read: Self::ReadData, _write: Self::WriteData) {

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
        EntityListener<Node, ModifyEvent>
        EntityListener<Node, DeleteEvent>
    }
}

fn main() { 

}