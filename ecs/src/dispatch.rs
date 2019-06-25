use atom::Atom;
use listener::{FnListeners, Listener};

use world::World;

pub trait Dispatcher {
    fn build(&mut self, names: String, world: &World);
    fn init(&mut self, names: Vec<Atom>, world: &World);
    fn run(&self);
}

#[derive(Default)]
pub struct SeqDispatcher {
    vec: FnListeners<()>,
}
/// TODO 先实现一个简单的顺序执行的派发器
impl Dispatcher for SeqDispatcher {
    fn build(&mut self, names: String, world: &World){
        let mut v = Vec::new();
        for s in names.split(',') {
            v.push(Atom::from(s.trim_start().trim_end()))
        }
        self.init(v, world);
    }
    fn init(&mut self, names: Vec<Atom>, world: &World) {
        // 简单实现
        for k in names.iter() {
            let sys = match world.get_system(&k) {
                Some(r) => r,
                None => panic!("system is not exist:{}", **k),
            };
            match sys.fetch_run() {
                Some(run) => self.vec.push_back(run),
                None => ()
            }
        }
        
        
        // 根据系统的读写数据，计算依赖关系。 如果一个数据被读写，则读会依赖写。写会先执行，读后执行
        // let mut system_map = FnvHashMap::default();
        // let mut component_map = FnvHashMap::default();
        //let mut vec = &mut self.vec;
        // for k in names.iter() {
        //     depend(world, k, &mut system_map, &mut component_map)
        // }
        // let mut len = names.len();
        // loop {
        //     for i in 0..len {
        //         if calc(&names[len - i - 1], &mut system_map, &mut component_map) {
        //             let key = names.swap_remove(len - i- 1);
        //             system_map.remove(&key);
        //             let sys = world.get_system(&key).unwrap();
        //             let run = sys.fetch_run(sys.clone(), world).unwrap();
        //             self.vec.push_back(run);
        //         }
        //     }
        //     if len == names.len() {
        //         panic!("cycle depend, {:?}", names);
        //     }
        //     len = names.len();
        //     if len == 0 {
        //         break;
        //     }
        // }
    }
    fn run(&self) {
        // let time = std::time::Instant::now();
        self.vec.listen(&());
        // println!("time----------{:?}", std::time::Instant::now() - time);
    }
}

//====================================
// // 根据系统的读写数据，计算依赖关系
// fn depend(world: &World, key: &Atom, system_map: &mut FnvHashMap<Atom, (Vec<(TypeId, TypeId)>, Vec<(TypeId, TypeId)>)>, component_map: &mut FnvHashMap<(TypeId, TypeId), (Vec<Atom>, Vec<Atom>)>) {
//     match world.get_system(key) {
//         Some(arc_sys) => {
//             let (read, write) = arc_sys.get_depends();
//             for r in read.iter() {
//                 match component_map.entry((r.0, r.1)) {
//                     Entry::Occupied(mut e) =>e.get_mut().0.push(key.clone()),
//                     Entry::Vacant(e) => {e.insert((vec![key.clone()], Vec::new()));}
//                 }
//             }
//             for r in write.iter() {
//                 match component_map.entry((r.0, r.1)) {
//                     Entry::Occupied(mut e) =>e.get_mut().1.push(key.clone()),
//                     Entry::Vacant(e) => {e.insert((Vec::new(), vec![key.clone()]));}
//                 }
//             }
//             system_map.insert(key.clone(), (read, write));
//         },
//         _ => ()
//     }
// }

// // 根据依赖关系，计算先后次序
// fn calc(key: &Atom, system_map: &FnvHashMap<Atom, (Vec<(TypeId, TypeId)>, Vec<(TypeId, TypeId)>)>, component_map: &FnvHashMap<(TypeId, TypeId), (Vec<Atom>, Vec<Atom>)>) -> bool {
//     let (read_components, _) = system_map.get(key).unwrap();
//     for k in read_components.iter() {
//         match component_map.get(k) {
//             Some((_, write_systems)) => {
//                 for w in write_systems.iter() {
//                     if system_map.get(w) != None {
//                         return false
//                     }
//                 }
//             },
//             _ => ()
//         }
//     }
//     true
// }