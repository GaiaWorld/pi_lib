use std::marker::PhantomData;
use std::io::{Error, Result, ErrorKind};
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};

use parking_lot::Mutex as SyncMutex;
use log::warn;

use pi_async::{lock::mutex_lock::Mutex,
              rt::{AsyncTaskPool, AsyncTaskPoolExt, AsyncRuntime}};

///
/// 异步二进制缓冲区回调结果
///
pub enum CallbackResult {
    Closed(String),     //关闭二进制缓冲区
    Continue(usize),    //继续运行二进制缓冲区，并重置缓冲区超时时长
}

///
/// 异步二进制缓冲区构建器
///
pub struct AsyncBytesBufferBuilder<
    O: Default + Send + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> {
    name:       String,         //缓冲区名称
    capacity:   usize,          //缓冲区容量
    timeout:    usize,          //缓冲区超时时长
    marker:     PhantomData<(O, P)>,
}

impl<
    O: Default + Send + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> AsyncBytesBufferBuilder<O, P> {
    /// 构建异步二进制缓冲区运行器，容量单位为byte，超时时长单位为ms
    pub fn new(name: &str,
               capacity: usize,
               timeout: usize) -> Self {
        AsyncBytesBufferBuilder {
            name: name.to_string(),
            capacity,
            timeout,
            marker: PhantomData,
        }
    }

    /// 构建异步二进制缓冲区，缓冲区会在指定的异步运行时中运行，每次运行会回调指定的回调函数
    pub fn build<RT, CC, TC>(self,
                            rt: RT,
                            capacity_callback: CC,
                            timeout_callback: TC) -> AsyncBytesBuffer<O, P, RT, CC>
        where RT: AsyncRuntime<O, Pool = P>,
              CC: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> usize + Send + Sync + 'static,
              TC: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> CallbackResult + Send + Sync + 'static {
        let rt_copy = rt.clone();
        let inner = InnerBytesBuffer {
            name: self.name.clone(),
            rt,
            buf: Mutex::new(Some(Vec::new())),
            size: AtomicUsize::new(0),
            capacity: AtomicUsize::new(self.capacity),
            callback: SyncMutex::new(capacity_callback),
            status: AtomicBool::new(true),
            marker: PhantomData,
        };
        let buffer = AsyncBytesBuffer(Arc::new(inner));
        let buffer_copy = buffer.clone();

        //在指定时间后运行一次二进制缓冲区
        AsyncBytesBufferBuilder::run_once(self,
                                          rt_copy,
                                          buffer,
                                          timeout_callback);

        buffer_copy
    }

    // 在指定时间后运行一次二进制缓冲区
    fn run_once<RT, CC, TC>(mut runner: AsyncBytesBufferBuilder<O, P>,
                            rt: RT,
                            buffer: AsyncBytesBuffer<O, P, RT, CC>,
                            mut timeout_callback: TC)
        where RT: AsyncRuntime<O, Pool = P>,
              CC: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> usize + Send + Sync + 'static,
              TC: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> CallbackResult + Send + Sync + 'static {
        let rt_copy = rt.clone();
        let timeout = runner.timeout;

        rt.spawn_timing(rt.alloc(), async move {
            let mut locked = buffer.0.buf.lock().await;
            if let Some(buf) = (&mut *locked).take() {
                //缓冲区存在，则调用用户指定的回调
                if buf.len() == 0 {
                    //缓冲区无内容，则在指定缓冲区超时时长后继续运行一次二进制缓冲区
                    buffer.0.size.store(0, Ordering::Relaxed); //重置缓冲区长度
                    AsyncBytesBufferBuilder::run_once(runner,
                                                      rt_copy,
                                                      buffer,
                                                      timeout_callback);
                } else {
                    //缓冲区有内容
                    let buf_size = buffer.0.size.load(Ordering::Relaxed);
                    match timeout_callback(buf, buf_size, timeout) {
                        CallbackResult::Closed(reason) => {
                            //关闭缓冲区
                            buffer.0.status.store(false, Ordering::SeqCst); //设置缓冲区为关闭状态
                            warn!("Close async buffer ok, name: {}, reason: {}", runner.name, reason);
                            return Default::default();
                        },
                        CallbackResult::Continue(new_timeout) => {
                            //在指定缓冲区超时时长后继续运行一次二进制缓冲区
                            runner.timeout = new_timeout; //重置缓冲区超时时长
                            buffer.0.size.store(0, Ordering::Relaxed); //重置缓冲区长度

                            AsyncBytesBufferBuilder::run_once(runner,
                                                              rt_copy,
                                                              buffer,
                                                              timeout_callback);
                        },
                    }
                }
            } else {
                //缓冲区不存在，则在指定缓冲区超时时长后继续运行一次二进制缓冲区
                buffer.0.size.store(0, Ordering::Relaxed); //重置缓冲区长度
                AsyncBytesBufferBuilder::run_once(runner,
                                                  rt_copy,
                                                  buffer,
                                                  timeout_callback);
            }

            //重置缓冲区
            *locked = Some(Vec::new());

            Default::default()
        }, timeout);
    }
}

///
/// 异步二进制缓冲区
///
pub struct AsyncBytesBuffer<
    O: Default + Send + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RT: AsyncRuntime<O, Pool = P>,
    F: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> usize + Send + Sync + 'static,
>(Arc<InnerBytesBuffer<O, P, RT, F>>);

unsafe impl<
    O: Default + Send + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
    RT: AsyncRuntime<O, Pool = P>,
    F: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> usize + Send + Sync + 'static,
> Send for AsyncBytesBuffer<O, P, RT, F> {}
unsafe impl<
    O: Default + Send + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
    RT: AsyncRuntime<O, Pool = P>,
    F: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> usize + Send + Sync + 'static,
> Sync for AsyncBytesBuffer<O, P, RT, F> {}

impl<
    O: Default + Send + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
    RT: AsyncRuntime<O, Pool = P>,
    F: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> usize + Send + Sync + 'static,
> Clone for AsyncBytesBuffer<O, P, RT, F> {
    fn clone(&self) -> Self {
        AsyncBytesBuffer(self.0.clone())
    }
}

/*
* 异步二进制缓冲区同步方法
*/
impl<
    O: Default + Send + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
    RT: AsyncRuntime<O, Pool = P>,
    F: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> usize + Send + Sync + 'static,
> AsyncBytesBuffer<O, P, RT, F> {
    /// 判断异步二进制缓冲区是否正在运行中
    pub fn is_running(&self) -> bool {
        self.0.status.load(Ordering::SeqCst)
    }

    /// 获取异步二进制缓冲区当前长度
    pub fn size(&self) -> usize {
        self.0.size.load(Ordering::Relaxed)
    }

    /// 同步非阻塞的向缓冲区推送二进制数据
    pub fn push(&self, bin: Arc<Vec<u8>>) {
        let buffer = self.clone();
        self.0.rt.spawn(self.0.rt.alloc(), async move {
            let _ = buffer.async_push(bin).await;
            Default::default()
        });
    }
}

/*
* 异步二进制缓冲区异步方法
*/
impl<
    O: Default + Send + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
    RT: AsyncRuntime<O, Pool = P>,
    F: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> usize + Send + Sync + 'static,
> AsyncBytesBuffer<O, P, RT, F> {
    /// 异步阻塞的向缓冲区推送二进制数据
    pub async fn async_push(&self, bin: Arc<Vec<u8>>) -> Result<()> {
        if !self.is_running() {
            //异步二进制缓冲区已关闭，则立即返回错误
            return Err(Error::new(ErrorKind::Interrupted, format!("Async push to buffer failed, name: {}, reason: already closed", self.0.name)));
        }

        let bin_size = bin.len();
        {
            let mut locked = self.0.buf.lock().await;
            let buf_size = if let Some(buf) = &mut *locked {
                buf.push(bin);
                self.0.size.fetch_add(bin_size, Ordering::Relaxed) + bin_size
            } else {
                //当前缓冲区为空，则立即返回错误
                panic!("Async push to buffer failed, name: {}, reason: buffer not exists", self.0.name);
            };

            let capacity = self.0.capacity.load(Ordering::Relaxed);
            if capacity <= buf_size {
                //缓冲区大小已达到缓冲区容量限制
                let mut buf = (&mut *locked).take().unwrap();
                {
                    let callback = &mut *self.0.callback.lock();
                    let new_capacity = callback(buf, buf_size, capacity); //调用用户指定的回调
                    self.0.size.store(0, Ordering::Relaxed); //重置缓冲区长度
                    self.0.capacity.store(new_capacity, Ordering::Relaxed); //重置缓冲区容量
                }

                //重置缓冲区
                *locked = Some(Vec::new());
            }
        }

        Ok(())
    }
}

// 内部二进制缓冲区
struct InnerBytesBuffer<
    O: Default + Send + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RT: AsyncRuntime<O, Pool = P>,
    F: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> usize + Send + Sync + 'static,
> {
    name:       String,                             //缓冲区名称
    rt:         RT,                                 //运行时
    buf:        Mutex<Option<Vec<Arc<Vec<u8>>>>>,   //缓冲区
    size:       AtomicUsize,                        //缓冲区长度
    capacity:   AtomicUsize,                        //缓冲区容量
    callback:   SyncMutex<F>,                       //回调函数
    status:     AtomicBool,                         //运行状态
    marker:     PhantomData<O>,
}

unsafe impl<
    O: Default + Send + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RT: AsyncRuntime<O, Pool = P>,
    F: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> usize + Send + Sync + 'static,
> Send for InnerBytesBuffer<O, P, RT, F> {}

unsafe impl<
    O: Default + Send + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RT: AsyncRuntime<O, Pool = P>,
    F: FnMut(Vec<Arc<Vec<u8>>>, usize, usize) -> usize + Send + Sync + 'static,
> Sync for InnerBytesBuffer<O, P, RT, F> {}