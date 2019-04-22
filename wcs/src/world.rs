use std::rc::Rc;

use fnv::FnvHashMap;

use atom::Atom;

impl<C: ComponentMgr, E> World<C, E> {
    pub fn new(mgr: C) -> World<C, E>{
        World{
            component_mgr : mgr,
            systems_mgr: FnvHashMap::default(),
            system_groups: FnvHashMap::default(),
            // systems: Vec::new(),
        }
    }

    pub fn add_systems<'a, L: Iterator<Item=&'a Atom>>(&mut self, name: Atom, list: &mut L) -> Result<(), String>{
        // debug版判断是否已经存在名为name的system_group， 如果存在， 输出警告 TODO
        let mut systems = Vec::new();
        for l in list {
            println!("systems:{:?}", l);
             match self.systems_mgr.get(l) {
                Some(v) => systems.push(v.clone()),
                None => return Err(format!("add_systems error, system is not exist, system_name: {}", l.as_ref())),
            };
        }
        self.system_groups.insert(name, systems);
        Ok(())
    }

    pub fn remove_systems(&mut self, name: &Atom){
        self.system_groups.remove(name);
    }

    pub fn register_system(&mut self, name: Atom, system: Rc<System<E, C>>){
        // debug版判断是否已经存在名为name的system， 如果存在， 输出警告 TODO
        self.systems_mgr.insert(name, system);
    }

    pub fn unregister_system(&mut self, name: &Atom) -> Option<Rc<System<E, C>>>{
        self.systems_mgr.remove(name)
    }

    pub fn run(&mut self, name: &Atom, e: E){
        let mut c_mgr = &mut self.component_mgr;
        let system_group = match self.system_groups.get(name) {
            Some(v) => v,
            None => {
                println!("run systems fail, it's bot exist, system_group_name: {}", name.as_ref());
                return;
            },
        };
        for runner in system_group.iter(){
            runner.run(&e, &mut c_mgr);
        }
    }
}

pub struct World<C: ComponentMgr, E>{
    pub component_mgr : C,
    systems_mgr: FnvHashMap<Atom, Rc<System<E, C>>>,
    system_groups: FnvHashMap<Atom, Vec<Rc<System<E, C>>>>

    // systems: Vec<Rc<System<E, C>>>,
}

impl<C: ComponentMgr + Default, E> Default for World<C, E> {
    fn default() -> Self {
        World{
            component_mgr: C::default(),
            systems_mgr: FnvHashMap::default(),
            system_groups: FnvHashMap::default()
            // systems: Vec::new(),
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
