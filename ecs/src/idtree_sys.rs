
use std::{
  marker::PhantomData,
};


use system::{SingleCaseListener};
use monitor::{DeleteEvent};
use single::SingleCaseImpl;
use idtree::IdTree;
use entity::EntityImpl;
use Share;

#[derive(Debug, Clone, Default)]
struct IdTreeSys<E>(PhantomData<E>);

impl<'a, T: Share, E: Share> SingleCaseListener<'a, IdTree<T>, DeleteEvent> for IdTreeSys<E> {
    type ReadData = &'a SingleCaseImpl<IdTree<T>>;
    type WriteData = &'a mut EntityImpl<E>;

    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData) {
      write.delete(event.id);
      for id in read.iter(event.id) {
        write.delete(id)
      }
    }
}
