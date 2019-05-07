
use std::{
    sync::Arc,
    any::{TypeId},
};

use world::World;
use listener::{Listener as Lis, FnListener, FnListeners};

pub trait Runner {
    type ReadData: FetchData;
    type WriteData: FetchMutData;

    fn setup(&mut self, read: Self::ReadData, write: Self::WriteData);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData);
    fn dispose(&mut self, read: Self::ReadData, write: Self::WriteData);
}

pub trait FetchData: Sized + 'static {
    fn fetch(world: &World) -> Self;
}

pub trait FetchMutData {
    fn fetch_mut(world: &World) -> Self;
}


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

/// E 是Entity的类型， 如果是单例组件， 则E为()。 C是组件类型， 如果仅监听Entity的创建和删除， 则C为()。 EV是事件类型
pub trait Listener<E, C, EV> {
    type ReadData: FetchData;
    type WriteData: FetchMutData;

    fn listen(&mut self, event: &EV, read: &Self::ReadData, write: &mut Self::WriteData);
}


pub type CreateListeners = FnListeners<CreateEvent>;
pub type DeleteListeners = FnListeners<DeleteEvent>;
pub type ModifyListeners = FnListeners<ModifyEvent>;
pub type CreateFn = FnListener<CreateEvent>;
pub type DeleteFn = FnListener<DeleteEvent>;
pub type ModifyFn = FnListener<ModifyEvent>;
pub type RunnerFn = FnListener<()>;


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

pub trait System {
    fn get_depends(&self) -> (Vec<(TypeId, TypeId)>, Vec<(TypeId, TypeId)>);
    fn fetch_setup(&self, me: Arc<System>, world: &World) -> Option<RunnerFn>;
    fn fetch_run(&self, me: Arc<System>, world: &World) -> Option<RunnerFn>;
    fn fetch_dispose(&self, me: Arc<System>, world: &World) -> Option<RunnerFn>;
}


// Node{};
// CharNode{};

// Pos{

// }

// pub struct Xy {};

// mod Xy{
//     const xx: HashMap<>;
// }

// struct CellXy = (TrustCell<Xy>);
// impl System for CellXy {
//     fn fetch_run(&self, me: Arc<Any>) -> Option<RunnerFn> {
//         let f = |e: &E| -> {
            
//             system.listen(e, &read_data, &mut write_data)
//         };
//         f
//     }
// }
// [#aa(dd)]
// impl Listener<T, Pos, CreateEvent> for Xy {
//     type ReadData = CellMultiCase<Node, WorldMatrix>;
//     type WriteData: Overflow;
//     fn listen(&mut self, event: &E, read: Self::ReadData, write: Self::WriteData) {

//     }
// }

// impl Listener<T, Pos, CreateEvent> for Xy {
//     install(world: &World) {
//         system;
//         let read_data = xxx.fetch(world: &World);
//         let write_data = xxx.fetch(world: &World);
//         let fn = |e: &E| -> {
//             system.listen(e, &read_data, &mut write_data)
//         };
//         let mut notify = world.get_notify<T, Pos>();
//         notify.create.push_back(Arc<fn>);
//     }
//     uninstall()
// }

// [#aa(dd)]
// impl Listener<T, Pos, DeleteEvent> for Xy {
//     type ReadData = MultiCase<Node, WorldMatrix>;
//     type WriteData: Overflow;
//     fn listen(&mut self, event: &E, read: Self::ReadData, write: Self::WriteData) {

//     }
// }