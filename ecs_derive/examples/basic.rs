extern crate ecs;
extern crate map;
extern crate hashmap;
#[macro_use]
extern crate ecs_derive;

use ecs::component::{ Component};
use map::vecmap::VecMap;
// use hashmap::HashMap;

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


#[derive(Write)]
pub struct Single{
    x: usize,
    y: usize,
}

#[derive(Write)]
pub struct Single1(String, f32);

pub struct Single2{
    x: usize,
    y: usize,
}

write!{
    pub struct Single2{
        x: usize,
        y: usize,
    }
}

pub struct Single3(String, f32);

write!{
    pub struct Single3(String, f32);
}

fn main() { 

}