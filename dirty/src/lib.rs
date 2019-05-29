use std::slice::Iter;


pub struct LayerDirty {
    dirtys: Vec<Vec<usize>>, // 按层放置的脏节点
    count: usize,            // 脏节点数量
    start: usize,            // 脏节点的起始层
}
impl Default for LayerDirty {
    fn default() -> LayerDirty {
        LayerDirty {
            dirtys: vec![Vec::new()],
            count: 0,
            start: usize::max_value(),
        }
    }
}
impl LayerDirty {
    // 设置节点脏
    pub fn mark(&mut self, id: usize, layer: usize) {
        self.count += 1;
        if self.start > layer {
            self.start = layer;
        }
        if self.dirtys.len() <= layer {
            for _ in self.dirtys.len()..layer + 1 {
                self.dirtys.push(Vec::new())
            }
        }
        let vec = unsafe { self.dirtys.get_unchecked_mut(layer) };
        vec.push(id);
    }
    pub fn delete(&mut self, id: usize, layer: usize) {
        let vec = unsafe { self.dirtys.get_unchecked_mut(layer) };
        for i in 0..vec.len() {
            if vec[i] == id {
                vec.swap_remove(i);
                self.count -= 1;
                break;
            }
        }
    }
    // 迭代方法
    pub fn iter(&self) -> DirtyIterator {
        if self.count == 0 {
            DirtyIterator {
                inner: self,
                layer: self.start,
                iter: self.dirtys[0].iter(),
            }
        } else {
             DirtyIterator {
                inner: self,
                layer: self.start + 1,
                iter: self.dirtys[self.start].iter(),
            }
        }
    }
    pub fn clear(&mut self) {
        let len = self.dirtys.len();
        while self.start < len {
            let vec = unsafe { self.dirtys.get_unchecked_mut(self.start) };
            let c = vec.len();
            self.start += 1;
            if c == 0 {
                continue;
            }
            self.count -= c;
            vec.clear();
        }
    }
}


pub struct DirtyIterator<'a> {
    inner: &'a LayerDirty,
    layer: usize,
    iter: Iter<'a, usize>,
}

impl<'a> Iterator for DirtyIterator<'a> {
    type Item = &'a usize;

    fn next(&mut self) -> Option<Self::Item> {
        let mut r = self.iter.next();
        if r == None {
            let len = self.inner.dirtys.len();
            while self.layer < len {
                let vec = unsafe { self.inner.dirtys.get_unchecked(self.layer) };
                self.layer += 1;
                if vec.len() > 0 {
                    self.iter = vec.iter();
                    r = self.iter.next();
                    break;
                }
            }
        }
        r
    }
}