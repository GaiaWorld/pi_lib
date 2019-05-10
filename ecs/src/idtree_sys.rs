
use std::{
  mem::{replace},
  marker::PhantomData,
};

use map::{Map, vecmap::VecMap};
use pointer::cell::TrustCell;
use Share;


use monitor::{Notify, NotifyImpl, CreateFn, DeleteFn, ModifyFn};
use single::SingleCase;
use idtree::IdTree;
use entity::Entity;

struct IdTreeSys;

// impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for IdTreeSys {
//     type ReadData = &'a IdTree;
//     type WriteData = &'a mut Entity;

//     fn listen(&mut self, _event: &DeleteEvent, _read: Self::ReadData, _write: Self::WriteData) {}
// }
