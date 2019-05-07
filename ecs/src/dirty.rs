std::slice::Iter;

use map::vecmap::VecMap;

pub struct Dirty {
    pub dirtys: Vec<usize>,
    pub dirty_mark_map: VecMap<bool>,
}

impl Dirty {
    pub fn new() -> Dirty{
        Dirty{
            dirtys: Vec::new(),
            dirty_mark_map: VecMap::new(),
        }
    }
    pub fn iter(&self) -> Iter<usize> {
        self.dirtys.iter()
    }
    pub fn marked(&mut self, id: usize){
        let dirty_mark = unsafe{self.dirty_mark_map.get_unchecked_mut(id)};
        if *dirty_mark == true {
            return;
        }
        *dirty_mark = true;
        self.dirtys.push(id);
    }
    pub fn delete(&mut self, id: usize){
        let dirty_mark = unsafe{self.dirty_mark_map.get_unchecked_mut(id)};
        *dirty_mark = false;
        for i in 0..self.dirtys.len(){
            if self.dirtys[i] == id{
                self.dirtys.remove(i);
                return;
            }
        }
    }
    pub fn clear(&mut self){
        self.dirtys.clear();
        self.dirty_mark_map.clear();
    }

}