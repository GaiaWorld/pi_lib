use std::{
    sync::Arc,
    mem::size_of,
};

use pointer::cell::TrustCell;
use slab::Slab;


use system::{Notify, NotifyImpl, CreateFn, DeleteFn, ModifyFn};
use component::MultiCase;

pub type CellEntity = TrustCell<Entity>;
impl Notify for CellEntity {
    fn add_create(&self, listener: CreateFn) {
        self.borrow_mut().notify.create.push_back(listener)
    }
    fn add_delete(&self, listener: DeleteFn) {
        self.borrow_mut().notify.delete.push_back(listener)
    }
    fn add_modify(&self, listener: ModifyFn) {
        self.borrow_mut().notify.modify.push_back(listener)
    }
    fn create_event(&self, id: usize) {
        self.borrow().notify.create_event(id);
    }
    fn delete_event(&self, id: usize) {
        self.borrow().notify.delete_event(id);
    }
    fn modify_event(&self, id: usize, field: &'static str, index: usize) {
        self.borrow().notify.modify_event(id, field, index);
    }
    fn remove_create(&self, listener: &CreateFn) {
        self.borrow_mut().notify.create.delete(listener);
    }
    fn remove_delete(&self, listener: &DeleteFn) {
        self.borrow_mut().notify.delete.delete(listener);
    }
    fn remove_modify(&self, listener: &ModifyFn) {
        self.borrow_mut().notify.modify.delete(listener);
    }
}
#[derive(Default)]
pub struct Entity{
    slab: Slab<usize>, // 值usize 记录每个id所关联的component的掩码位
    components: Vec<Arc<MultiCase>>, // 组件
    notify: NotifyImpl,
}
impl Entity {
    pub fn get_mask(&self) -> usize {
        self.components.len()
    }
    pub fn register_component(&mut self, component: Arc<MultiCase>) {
        self.components.push(component);
    }
    pub fn create(&mut self) -> usize {
        let id = self.slab.insert(0);
        self.notify.create_event(id);
        id
    }
    pub fn delete(&mut self, id: usize) {
        let mask = self.slab.remove(id);
        self.notify.delete_event(id);
        if mask == 0 {
            return
        }
        // 依次删除对应的组件
        for i in mask.trailing_zeros() as usize..size_of::<usize>()-(mask.leading_zeros() as usize)+1 {
            if mask & (1<<i) != 0 {
                self.components[i].delete(id)
            }
        }
    }

}

