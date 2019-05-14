
use std::{
    any::{TypeId},
    sync::Arc,
};
use { World, Fetch, TypeIds};
pub use listener::FnListener;

pub trait Runner<'a> {
    type ReadData: SystemData<'a>;
    type WriteData: SystemMutData<'a>;

    fn setup(&mut self, _read: Self::ReadData, _write: Self::WriteData){}
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData);
    fn dispose(&mut self, _read: Self::ReadData, _write: Self::WriteData){}
}

pub trait SystemData<'a> where Self: std::marker::Sized{
    type FetchTarget: Fetch + TypeIds;
}

pub trait SystemMutData<'a> where Self: std::marker::Sized{
    type FetchTarget: Fetch + TypeIds;
}

/// E 是Entity的类型， C是组件类型， EV是事件类型
pub trait MultiCaseListener<'a, E, C, EV> {
    type ReadData: SystemData<'a>;
    type WriteData: SystemMutData<'a>;

    fn listen(&mut self, event: &EV, read: Self::ReadData, write: Self::WriteData);
}

/// Entity监听器， 监听Entity的创建和删除， EV是事件类型
pub trait EntityListener<'a, E, EV> {
    type ReadData: SystemData<'a>;
    type WriteData: SystemMutData<'a>;

    fn elisten(&mut self, event: &EV, read: Self::ReadData, write: Self::WriteData);
}
/// 单例组件监听器， EV是事件类型
pub trait SingleCaseListener<'a, C, EV> {
    type ReadData: SystemData<'a>;
    type WriteData: SystemMutData<'a>;

    fn slisten(&mut self, event: &EV, read: Self::ReadData, write: Self::WriteData);
}

pub type RunnerFn = FnListener<()>;
pub type DisposeFn = FnListener<World>;

pub trait System: any::ArcAny { 
    fn setup(&mut self, me: Arc<System>, world: &World);
    fn dispose(&self, world: &World);
    fn fetch_run(&self) -> Option<RunnerFn>;
    fn get_depends(&self) -> (Vec<(TypeId, TypeId)>, Vec<(TypeId, TypeId)>);
}
impl_downcast_arc!(System);

#[macro_export(local_inner_macros)]
macro_rules! impl_system {
    (@add_monitor $setup_target:ident, $f:ident, $e:ty, $c:ty, CreateEvent) => {$crate::monitor::Notify::add_create(&*$setup_target, $f.clone())};
    (@add_monitor $setup_target:ident, $f:ident, $ec:ty, CreateEvent) => {$crate::monitor::Notify::add_create(&*$setup_target, $f.clone())};
    (@add_monitor $setup_target:ident, $f:ident, $e:ty, $c:ty, ModifyEvent) => {$crate::monitor::Notify::add_modify(&*$setup_target, $f.clone())};
    (@add_monitor $setup_target:ident, $f:ident, $ec:ty, ModifyEvent) => {$crate::monitor::Notify::add_modify(&*$setup_target, $f.clone())};
    (@add_monitor $setup_target:ident, $f:ident, $e:ty, $c:ty, DeleteEvent) => {$crate::monitor::Notify::add_delete(&*$setup_target, $f.clone())};
    (@add_monitor $setup_target:ident, $f:ident, $ec:ty, DeleteEvent) => {$crate::monitor::Notify::add_delete(&*$setup_target, $f.clone())};

    (@remove_monitor $setup_target:ident, $f:expr, $e:ty, $c:ty, CreateEvent) => {
        let r: Box<Fn(&CreateEvent)> = unsafe {std::mem::transmute($f.clone())};
        let r: $crate::monitor::FnListener<CreateEvent> = $crate::monitor::FnListener(unsafe{std::sync::Arc::from_raw(Box::into_raw(r))});
        $crate::monitor::Notify::remove_create(&*$setup_target, &r);
    };
    (@remove_monitor $setup_target:ident, $f:expr, $ec:ty, CreateEvent) => {
        let r: Box<Fn(&CreateEvent)> = unsafe {std::mem::transmute($f.clone())};
        let r: $crate::monitor::FnListener<CreateEvent> = $crate::monitor::FnListener(unsafe{std::sync::Arc::from_raw(Box::into_raw(r))});
        $crate::monitor::Notify::remove_create(&*$setup_target, &r)
    };
    (@remove_monitor $setup_target:ident, $f:expr, $e:ty, $c:ty, ModifyEvent) => {
        let r: Box<Fn(&ModifyEvent)> = unsafe {std::mem::transmute($f.clone())};
        let r: $crate::monitor::FnListener<ModifyEvent> = $crate::monitor::FnListener(unsafe{std::sync::Arc::from_raw(Box::into_raw(r))});
        $crate::monitor::Notify::remove_modify(&*$setup_target, &r)
    };
    (@remove_monitor $setup_target:ident, $f:expr, $ec:ty, ModifyEvent) => {
        let r: Box<Fn(&ModifyEvent)> = unsafe {std::mem::transmute($f.clone())};
        let r: $crate::monitor::FnListener<ModifyEvent> = $crate::monitor::FnListener(unsafe{std::sync::Arc::from_raw(Box::into_raw(r))});
        $crate::monitor::Notify::remove_modify(&*$setup_target, &r)
    };
    (@remove_monitor $setup_target:ident, $f:expr, $e:ty, $c:ty, DeleteEvent) => {
        let r: Box<Fn(&DeleteEvent)> = unsafe {std::mem::transmute($f.clone())};
        let r: $crate::monitor::FnListener<DeleteEvent> = $crate::monitor::FnListener(unsafe{std::sync::Arc::from_raw(Box::into_raw(r))});
        $crate::monitor::Notify::remove_delete(&*$setup_target, &r)
    };
    (@remove_monitor $setup_target:ident, $f:expr, $ec:ty, DeleteEvent) => {
        let r: Box<Fn(&DeleteEvent)> = unsafe {std::mem::transmute($f.clone())};
        let r: $crate::monitor::FnListener<DeleteEvent> = $crate::monitor::FnListener(unsafe{std::sync::Arc::from_raw(Box::into_raw(r))});
        $crate::monitor::Notify::remove_delete(&*$setup_target, &r)
    };

    // fetch_single fetch_multi fetch_entry
    (@setup_target_ty $setup_target:ident, $w:ident, SingleCaseListener, $c:ty, $ev:ty) => {
        let $setup_target = $w.fetch_single::<$c>().unwrap();
    };
    (@setup_target_ty $setup_target:ident, $w:ident, MultiCaseListener, $e:ty, $c:ty, $ev:ty) => {
        let $setup_target = $w.fetch_multi::<$e, $c>().unwrap();
    };
    (@setup_target_ty $setup_target:ident, $w:ident, EntityListener, $e:ty, $ev:ty) => {
        let $setup_target = $w.fetch_entity::<$e>().unwrap();
    };

    // 
    (@call_listen $e1:ident, $me1: ident, $read_data:ident, $write_data:ident, SingleCaseListener, $c:ty, $ev:ty) => {
        $me1.owner.borrow_mut().slisten($e1, $read_data, $write_data);
    };
    (@call_listen $e1:ident, $me1: ident, $read_data:ident, $write_data:ident, MultiCaseListener, $e:ty, $c:ty, $ev:ty) => {
        $me1.owner.borrow_mut().listen($e1, $read_data, $write_data);
    };
    (@call_listen $e1:ident, $me1: ident, $read_data:ident, $write_data:ident, EntityListener, $e:ty, $ev:ty) => {
        $me1.owner.borrow_mut().elisten($e1, $read_data, $write_data);
    };
    
    //每一个listenner setup
    (@listener_setup $arr:ident $world:ident $me:ident $system: tt <$($sg:ty),*>, $sign:tt < $($gen:tt),* > $($t:tt)* ) => {
        let me1 = $me.clone();
        let read = <<<$system <$($sg),*> as $crate::system::$sign<'_, $($gen),*>>::ReadData as $crate::system::SystemData>::FetchTarget as $crate::Fetch>::fetch($world);
        let write = <<<$system <$($sg),*> as $crate::system::$sign<'_, $($gen),*>>::WriteData as $crate::system::SystemMutData>::FetchTarget as  $crate::Fetch>::fetch($world);
        let f = $crate::monitor::FnListener(std::sync::Arc::new( move |e| {
            
            let read_data = $crate::Borrow::borrow(&read);
            let write_data = $crate::BorrowMut::borrow_mut(&write);
            impl_system!(@call_listen e, me1, read_data, write_data, $sign, $($gen),*);
        }));
        impl_system!(@setup_target_ty setup_target, $world, $sign, $($gen),*);
        impl_system!(@add_monitor setup_target, f, $($gen),*);
        let ptr: (usize, usize) = unsafe {std::mem::transmute(std::sync::Arc::into_raw(f.0))};
        $arr.push(ptr); // 裸指针
        impl_system!(@listener_setup $arr $world $me $system <$($sg),*>, $($t)*);
    };
    (@listener_setup $arr:ident $world:ident $me:ident $system: tt <$($sg:ty),*>,) => {};

    //每一个listenner dispose
    (@listener_dispose $i:expr; $f:ident $world:ident $me:ident $system: tt <$($sg:ty),*>, $sign:tt < $($gen:tt),* > $($t:tt)* ) => {
        impl_system!(@setup_target_ty setup_target, $world, $sign, $($gen),*);
        impl_system!(@remove_monitor setup_target, &$f[$i], $($gen),*); 
        impl_system!(@listener_dispose $i+1; $f $world $me $system <$($sg),*>, $($t)*);
    };
    (@listener_dispose $i:expr; $f:ident $world:ident $me:ident $system: tt <$($sg:ty),*>,) => {};

    //每一个listenner get_depends
    (@listener_get_depends $read_ids:ident $write_ids:ident $system: tt <$($sg:ty),*>, $sign:tt <$($gen:ty),*> $($t:tt)*) => {
        let r_ids = <<<$system <$($sg),*> as $crate::system::$sign<'_, $($gen),*>>::ReadData as $crate::system::SystemData>::FetchTarget as $crate::TypeIds>::type_ids();
        let w_ids = <<<$system <$($sg),*> as $crate::system::$sign<'_, $($gen),*>>::WriteData as $crate::system::SystemMutData>::FetchTarget as $crate::TypeIds>::type_ids();
        $read_ids.extend_from_slice(&r_ids);
        $write_ids.extend_from_slice(&w_ids);
        impl_system!(@listener_get_depends $read_ids $write_ids $system <$($sg),*>, $($t)*);
    };
    (@listener_get_depends $read_ids:ident $write_ids:ident $system: tt <$($sg:ty),*>,) => {};
    
    //每一个runner get_depends
    (@runner_get_depends $read_ids:ident $write_ids:ident $system: tt <$($sg:ty),*>, true) => {
        let r_ids = <<<$system <$($sg),*> as Runner>::ReadData as $crate::system::SystemData>::FetchTarget as $crate::TypeIds>::type_ids();
        let w_ids = <<<$system <$($sg),*> as Runner>::WriteData as $crate::system::SystemMutData>::FetchTarget as $crate::TypeIds>::type_ids();
        $read_ids.extend_from_slice(&r_ids);
        $write_ids.extend_from_slice(&w_ids);
    };
    (@runner_get_depends $read_ids:ident $write_ids:ident $system: tt <$($sg:ty),*>, false) => {}; // 如果没有实现runner，不需要取type_ids

    //runner setup
    (@runner_setup $s:ident $world:ident $me:ident $system: tt <$($sg:ty),*>, true) => {
        let read = <<<$system <$($sg),*> as $crate::system::Runner>::ReadData as $crate::system::SystemData>::FetchTarget as $crate::Fetch>::fetch($world);
        let write = <<<$system <$($sg),*> as $crate::system::Runner>::WriteData as $crate::system::SystemMutData>::FetchTarget as $crate::Fetch>::fetch($world);
        {
            let read_data = $crate::Borrow::borrow(&read);
            let write_data = $crate::BorrowMut::borrow_mut(&write);
            $s.owner.borrow_mut().setup(read_data, write_data);
        }
        $s.run_fn = Some($crate::monitor::FnListener(std::sync::Arc::new( move |e: &()| {
            let read_data = $crate::Borrow::borrow(&read);
            let write_data = $crate::BorrowMut::borrow_mut(&write);
            $me.owner.borrow_mut().run(read_data, write_data);
        })))
    };
    (@runner_setup $world:ident $me:ident $system: tt <$($sg:ty),*>, false) => {};

    //runner dispose
    (@runner_dispose $s:ident $world:ident $system: tt <$($sg:ty),*>, true) => {
        let read = <<<$system <$($sg),*> as $crate::system::Runner>::ReadData as $crate::system::SystemData>::FetchTarget as $crate::Fetch>::fetch($world);
        let write = <<<$system <$($sg),*> as $crate::system::Runner>::WriteData as $crate::system::SystemMutData>::FetchTarget as $crate::Fetch>::fetch($world);
        let read_data = $crate::Borrow::borrow(&read);
        let write_data = $crate::BorrowMut::borrow_mut(&write);
        $s.owner.borrow_mut().dispose(read_data, write_data);
        // $s.run_fn = None;
    };
    (@runner_dispose $world:ident $me:ident $system: tt <$($sg:ty),*>, false) => {};

    ($system:tt, $has_runner: tt, {$($t: tt)*}) => {impl_system!($system<>, $has_runner, {$($t)*} );};
    ($system:tt<$($sg:ty),*>, $has_runner: tt, {$($t: tt)*}) => {
        $crate::paste::item! {
            pub struct [<Cell $system>] <$($sg),*> {
                owner: pointer::cell::TrustCell<$system<$($sg),*>>,
                run_fn: Option<$crate::system::RunnerFn>,
                dispose_listener_fn: Option<$crate::system::DisposeFn>,
            }
            impl<$($sg),*> [<Cell $system>]<$($sg),*> {
                fn new(sys: $system<$($sg),*>) -> Self{
                    Self {
                        owner: pointer::cell::TrustCell::new(sys),
                        run_fn: None,
                        dispose_listener_fn: None,
                    }
                }
            }
        
            impl<$($sg),*> $crate::system::System for [<Cell $system>]<$($sg),*> {
                fn get_depends(&self) -> (Vec<(std::any::TypeId, std::any::TypeId)>, Vec<(std::any::TypeId, std::any::TypeId)>) {
                    let mut read_ids = Vec::new();
                    let mut write_ids = Vec::new();

                    //listeners depends
                    impl_system!(@listener_get_depends read_ids write_ids $system <$($sg),*>, $($t)*);

                    //runner depends
                    impl_system!(@runner_get_depends read_ids write_ids $system <$($sg),*>, $has_runner);

                    (read_ids, write_ids)
                }

                fn setup(&mut self, me: std::sync::Arc<$crate::system::System>, world: &$crate::world::World){
                    let me: std::sync::Arc<Self> = match $crate::system::System::downcast(me) {
                        Ok(r) => r,
                        Err(_) => std::panic!("downcast err".to_string()),
                    };

                    let mut listen_arr = Vec::new();
                    //listen setup
                    impl_system!(@listener_setup listen_arr world me $system <$($sg),*>, $($t)*);

                    //runner setup
                    impl_system!(@runner_setup self world me $system <$($sg),*>, $has_runner);

                    //dispose
                    self.dispose_listener_fn = Some($crate::monitor::FnListener(std::sync::Arc::new(move |world: &$crate::world::World| {
                        impl_system!(@listener_dispose 0; listen_arr world me $system <$($sg),*>, $($t)*);
                    })));
                }

                fn dispose(&self, world: &$crate::world::World) {
                    match &self.dispose_listener_fn {
                        Some(f) => f.0(world),
                        None => (),
                    };
                    // self.dispose_listener_fn = None;

                    // runner dispose
                    impl_system!(@runner_dispose self world $system <$($sg),*>, $has_runner);
                }

                fn fetch_run(&self) -> Option<$crate::system::RunnerFn> {
                    self.run_fn.clone()
                }
            }
        }
    };
}

macro_rules! impl_data {
    ( $($ty:ident),* ) => {
        impl<'a, $($ty),*> SystemData<'a> for ( $( $ty , )* ) where $( $ty : SystemData<'a> ),*{
            type FetchTarget = ($($ty::FetchTarget,)*);
        }

        impl<'a, $($ty),*> SystemMutData<'a> for ( $( $ty , )* ) where $( $ty : SystemMutData<'a> ),*{
            type FetchTarget = ($($ty::FetchTarget,)*);
        } 
    };
}

impl_data!();
impl_data!(A);
impl_data!(A, B);
impl_data!(A, B, C);
impl_data!(A, B, C, D);
impl_data!(A, B, C, D, E);
impl_data!(A, B, C, D, E, F);
impl_data!(A, B, C, D, E, F, G);
impl_data!(A, B, C, D, E, F, G, H);
impl_data!(A, B, C, D, E, F, G, H, I);
impl_data!(A, B, C, D, E, F, G, H, I, J);
impl_data!(A, B, C, D, E, F, G, H, I, J, K);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
