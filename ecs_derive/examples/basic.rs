extern crate ecs;
extern crate map;
#[macro_use]
extern crate ecs_derive;

use ecs::component::{ Component};
use map::vecmap::VecMap;

#[derive(Component)]
pub struct Position{
    x: usize,
    y: usize,
}

#[derive(Component)]
pub struct ZIndex(usize);

pub struct ZIndex1(ZIndex);

component!{
    struct ZIndex1(usize);
}

pub struct Position1(Position);

component!{
    struct Position1{
        x:usize,
        y:usize,
    }
}

fn main() { 

}