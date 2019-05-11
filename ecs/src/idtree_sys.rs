
use std::{
  marker::PhantomData,
};

use system::{EntityListener};
use monitor::{DeleteEvent};
use single::SingleCaseImpl;
use idtree::IdTree;
use entity::EntityImpl;
use Share;

#[derive(Debug, Clone, Default)]
struct IdTreeSys<E>(PhantomData<E>);

impl<'a, T: Share, E: Share> EntityListener<'a, E, DeleteEvent> for IdTreeSys<E> {
    type ReadData = &'a SingleCaseImpl<IdTree<T>>;
    type WriteData = &'a mut EntityImpl<E>;

    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData) {
      write.delete(event.id);
      for id in read.iter(event.id) {
        write.delete(id)
      }
    }
}
// impl_system!{
//     IdTreeSys<E>,
//     false,
//     {
//         SingleCaseListener<IdTree<T>, DeleteEvent>
//     }
// }