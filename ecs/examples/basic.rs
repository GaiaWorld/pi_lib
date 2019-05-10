#![feature(proc_macro_hygiene)]

///一个基本的例子， 定义组件， 实体， 系统， 已经如何实例化World并运行（TODO）

#[macro_use]
extern crate ecs;
extern crate map;
extern crate pointer;

use ecs::component::{ Component, MultiCaseImpl};
use ecs::system::{Runner, MultiCaseListener};
use ecs::monitor::{CreateEvent};
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

    fn setup(&mut self, _read: Self::ReadData, _write: Self::WriteData) {}
    fn run(&mut self, _read: Self::ReadData, _write: Self::WriteData) {}
    fn dispose(&mut self, _read: Self::ReadData, _write: Self::WriteData) {}
}

impl<'a> MultiCaseListener<'a, Node, Position, CreateEvent> for SystemDemo {
    type ReadData = &'a MultiCaseImpl<Node, Position>;
    type WriteData = &'a mut MultiCaseImpl<Node, Position>;

    fn listen(&mut self, _event: &CreateEvent, _read: Self::ReadData, _write: Self::WriteData) {}
}


impl_system!{
    SystemDemo,
    true,
    {
        MultiCaseListener<Node, Position, CreateEvent>
    }
}

fn main() { 
    
}