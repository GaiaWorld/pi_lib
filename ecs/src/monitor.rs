use listener::{Listener as LibListener, FnListeners};
pub use listener::FnListener;

pub struct CreateEvent{
    pub id: usize,
}

pub struct DeleteEvent{
    pub id: usize,
}

pub struct ModifyEvent{
    pub id: usize,
    pub field: &'static str,
    pub index: usize, // 一般无意义。 只有在数组或向量的元素被修改时，才有意义
}

pub type CreateListeners = FnListeners<CreateEvent>;
pub type DeleteListeners = FnListeners<DeleteEvent>;
pub type ModifyListeners = FnListeners<ModifyEvent>;
pub type CreateFn = FnListener<CreateEvent>;
pub type DeleteFn = FnListener<DeleteEvent>;
pub type ModifyFn = FnListener<ModifyEvent>;

#[derive(Default)]
pub struct NotifyImpl {
    pub create: CreateListeners,
    pub delete: DeleteListeners,
    pub modify: ModifyListeners,
}
impl NotifyImpl {
    pub fn create_event(&self, id: usize) {
        let e = CreateEvent{
            id: id,
        };
        self.create.listen(&e);
    }
    pub fn delete_event(&self, id: usize) {
        let e = DeleteEvent{
            id: id,
        };
        self.delete.listen(&e);
    }
    pub fn modify_event(&self, id: usize, field: &'static str, index: usize) {
        let e = ModifyEvent{
            id: id,
            field: field,
            index: index,
        };
        self.modify.listen(&e);
    }
}

pub trait Notify {
    fn add_create(&self, CreateFn);
    fn add_delete(&self, DeleteFn);
    fn add_modify(&self, ModifyFn);
    fn create_event(&self, id: usize);
    fn delete_event(&self, id: usize);
    fn modify_event(&self, id: usize, field: &'static str, index: usize);
    fn remove_create(&self, &CreateFn);
    fn remove_delete(&self, &DeleteFn);
    fn remove_modify(&self, &ModifyFn);
}

pub struct Write<'a, T>{
    pub id: usize,
    pub value: &'a mut T,
    pub notify: &'a NotifyImpl,
}

impl<'a, T> Write<'a, T> {
    pub fn new(id: usize, value: &'a mut T, notify: &'a NotifyImpl) -> Write<'a, T>{
        Write {
            id, value, notify
        }
    }
}