
use fnv::FnvHashMap;

use atom::Atom;
use listener::{FnListeners, Listener};

use world::World;

pub trait Dispatcher {
    fn add(&mut self, name: &Atom, depends: String);
    fn over(&mut self, world: &World);
    fn run(&self);
}

#[derive(Default)]
pub struct SeqDispatcher {
    map: FnvHashMap<Atom, Vec<Atom>>,
    vec: FnListeners<()>,
}

/// TODO 先实现一个简单的顺序执行的派发器
impl Dispatcher for SeqDispatcher {
    fn add(&mut self, name: &Atom, depends: String){
        let mut v = Vec::new();
        for s in depends.split(',') {
            v.push(Atom::from(s.trim_start().trim_end()))
        }
    }
    fn over(&mut self, world: &World) {
        //world.fetch_system();
    }
    fn run(&self) {
        self.vec.listen(&())
    }
}
