use std::sync::Arc;
use std::panic::resume_unwind;
use std::marker::{Send, Sync};

use futures::*;
use npnc::ConsumeError;
use npnc::bounded::spsc::{Producer, Consumer};

use time::now_millisecond;

///
/// 未来任务
///
#[derive(Debug)]
pub struct FutTask<T, E> {
    uid:        usize,                          //未来任务id
    timeout:    i64,                            //未来任务超时时间
    inner:      Arc<Consumer<Result<T, E>>>,    //内部未来任务
    sender:     Arc<Producer<task::Task>>,      //内部任务发送者
}

unsafe impl<T: Send + 'static, E: Send + 'static> Send for FutTask<T, E> {}
unsafe impl<T: Send + 'static, E: Send + 'static> Sync for FutTask<T, E> {}

impl<T: Send + 'static, E: Send + 'static> FutTask<T, E> {
    /// 构建一个未来任务
    pub fn new(uid: usize, timeout: u32, inner: Arc<Consumer<Result<T, E>>>, sender: Arc<Producer<task::Task>>) -> Self {
        FutTask {
            uid: uid,
            timeout: now_millisecond() as i64 + timeout as i64,
            inner: inner,
            sender: sender,
        }
    }

    /// 获取当前未来任务id
    pub fn get_uid(&self) -> usize {
        self.uid
    }
}

impl<T: Send + 'static, E: Send + 'static> Future for FutTask<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<T, E> {
        if self.timeout < now_millisecond() as i64 {
            resume_unwind(Box::new("future task timeout")) //超时
        } else {
            match self.inner.consume() {
                Ok(Ok(r)) => Ok(Async::Ready(r)),
                Ok(Err(e)) => Err(e),
                Err(e) => {
                    match e {
                        ConsumeError::Empty => {
                            //还未准备好
                            if self.sender.len() > 0 {
                                //忽略重复未准备好
                                return Ok(Async::NotReady);
                            }

                            match self.sender.produce(futures::task::current()) {
                                Err(e) => resume_unwind(Box::new(e.to_string())),
                                Ok(_) => Ok(Async::NotReady),
                            }
                        },
                        _ => resume_unwind(Box::new("future task failed")), //异常
                    }
                },
            }
        }
    }
}