/**
 * 按照层级关系记录node的脏标志
 */
use map::vecmap::{VecMap};

// use world_doc::WorldDocMgr;

#[derive(Default)]
pub struct LayerDirtyMark {
    pub dirtys: Vec<Vec<usize>>, //Vec<Vec<node_id>>, 脏节点
    pub count: usize, //脏节点数量
    pub start_layer: usize, //脏节点的起始层 
}

impl LayerDirtyMark {
    pub fn new() -> LayerDirtyMark{
        // 默认id为1的node为根， 根的创建没有事件， 因此默认插入根的脏
        let mut dirtys = Vec::new();

        LayerDirtyMark{
            dirtys,
            count: 1,
            start_layer: 0,
        }
    }

    pub fn marked_dirty(&mut self, id: usize, layer: usize) {
        let layer = layer - 1;
        if self.start_layer > layer {
            self.start_layer = layer;
        }

        if self.dirtys.len() <= layer{
            for _i in 0..(layer + 1 - self.dirtys.len()){
                self.dirtys.push(Vec::new());
            }
        }
        self.dirtys[layer].push(id);
        self.count += 1;
    }

    pub fn delete_dirty(&mut self, id: usize, layer: usize){
        let layer = layer - 1;
        let vec = &mut self.dirtys[layer];
        for i in 0..vec.len() {
            if vec[i] == id {
                vec.swap_remove(i);
                self.count -= 1;
                break;
            }
        }
    }

    pub fn iter(&mut self) -> Iter {
        Iter {
            layer:self.start_layer,
            index: 0,
            count: self.count,
            dirtys: &self.dirtys,
        }
    }

    pub fn clear(&mut self){
        for i in self.start_layer..self.dirtys.len() {
            self.dirtys[i].clear();
        }
        self.start_layer = 0;
        self.count = 0;
    }
}

pub struct Iter<'a>{
    layer: usize,
    index: usize,
    count: usize,
    dirtys: &'a Vec<Vec<usize>>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.count == 0 {
            return None;
        }
        let mut layer = self.layer;
        loop {
            
            let vec = &self.dirtys[layer];
            if vec.len() <= self.index {
                layer += 1;
                self.index = 0;
                continue;
            }
            let id = vec[self.index];

            self.index += 1;
            if self.index == self.dirtys[layer].len() {
                self.index = 0;
                layer += 1;
            }

            self.layer = layer;
            self.count -=1;
            return Some(id);
        }
    }
}

