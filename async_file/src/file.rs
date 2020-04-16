use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::future::Future;
use std::cell::{RefCell, Ref};
use std::path::{Path, PathBuf};
#[cfg(any(unix))]
use std::os::unix::fs::FileExt;
#[cfg(any(windows))]
use std::os::windows::fs::FileExt;
use std::task::{Context, Poll, Waker};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::fs::{File, OpenOptions, Metadata,
              rename as sync_rename,
              create_dir_all as sync_create_dir_all,
              remove_file as sync_remove_file,
              copy as sync_copy,
              remove_dir as sync_remove_dir};
use std::io::{Seek, Write, Result, SeekFrom, Error, ErrorKind};

use parking_lot::RwLock;
use r#async::rt::multi_thread::{MultiTaskPool, MultiTaskRuntime};

/*
* 异步重命名指定的文件的结果
*/
#[derive(Clone)]
struct RenameFileResult(Arc<RefCell<Option<Result<()>>>>);

unsafe impl Send for RenameFileResult {}
unsafe impl Sync for RenameFileResult {}

/*
* 异步重命名指定的文件或目录
*/
struct AsyncRenameFile<P: AsRef<Path> + Send + 'static, O: Default + 'static> {
    runtime:    MultiTaskRuntime<O>,    //异步运行时
    from:       P,                      //源文件路径
    to:         P,                      //目标文件路径
    result:     RenameFileResult,       //重命名文件结果
}

unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Send for AsyncRenameFile<P, O> {}
unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Sync for AsyncRenameFile<P, O> {}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Future for AsyncRenameFile<P, O> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.as_ref().result.0.borrow_mut().take() {
            //已重命名指定文件或目录，则返回
            return Poll::Ready(result);
        }

        //异步重命名指定文件或目录
        let task_id = self.as_ref().runtime.alloc();
        let runtime = self.as_ref().runtime.clone();
        let from = self.as_ref().from.as_ref().to_path_buf();
        let to = self.as_ref().to.as_ref().to_path_buf();
        let result = self.as_ref().result.clone();
        let task = async move {
            match sync_rename(from, to) {
                Err(e) => {
                    //异步重命名文件或目录失败，则设置等待异步重命名文件或目录的任务的值
                    *result.0.borrow_mut() = Some(Err(e));
                },
                Ok(_) => {
                    //异步重命名文件或目录成功，则设置等待异步重命名文件或目录的任务的值
                    *result.0.borrow_mut() = Some(Ok(()));
                },
            }

            //唤醒等待异步重命名文件或目录的任务
            runtime.wakeup(task_id);

            //返回当前异步任务的默认值
            Default::default()
        };
        if let Err(e) = self.as_ref().runtime.spawn(task_id, task) {
            //派发异步重命名文件或目录的任务失败，则立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Rename File or Dir Error, from: {:?}, to: {:?}, reason: {:?}", self.as_ref().from.as_ref(), self.as_ref().to.as_ref(), e))));
        }

        //挂起当前任务，并返回值未就绪
        self.as_ref().runtime.pending(task_id, cx.waker().clone())
    }
}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> AsyncRenameFile<P, O> {
    //构建异步重命名指定文件或目录的方法
    pub fn new(runtime: MultiTaskRuntime<O>, from: P, to: P) -> Self {
        AsyncRenameFile {
            runtime,
            from,
            to,
            result: RenameFileResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

/*
* 异步重命名文件或目录
*/
pub async fn rename<P, O>(runtime: MultiTaskRuntime<O>, from: P, to: P) -> Result<()>
    where P: AsRef<Path> + Send + 'static, O: Default + 'static {
    AsyncRenameFile::new(runtime, from, to).await
}

/*
* 异步创建指定的目录的结果
*/
#[derive(Clone)]
struct CreateDirResult(Arc<RefCell<Option<Result<()>>>>);

unsafe impl Send for CreateDirResult {}
unsafe impl Sync for CreateDirResult {}

/*
* 异步创建指定的目录
*/
struct AsyncCreateDir<P: AsRef<Path> + Send + 'static, O: Default + 'static> {
    runtime:    MultiTaskRuntime<O>,    //异步运行时
    path:       P,                      //文件路径
    result:     CreateDirResult,        //创建目录结果
}

unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Send for AsyncCreateDir<P, O> {}
unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Sync for AsyncCreateDir<P, O> {}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Future for AsyncCreateDir<P, O> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.as_ref().result.0.borrow_mut().take() {
            //已创建指定目录，则返回
            return Poll::Ready(result);
        }

        //异步创建指定目录
        let task_id = self.as_ref().runtime.alloc();
        let runtime = self.as_ref().runtime.clone();
        let path = self.as_ref().path.as_ref().to_path_buf();
        let result = self.as_ref().result.clone();
        let task = async move {
            match sync_create_dir_all(path) {
                Err(e) => {
                    //异步创建目录失败，则设置等待异步创建目录的任务的值
                    *result.0.borrow_mut() = Some(Err(e));
                },
                Ok(_) => {
                    //异步创建目录成功，则设置等待异步创建目录的任务的值
                    *result.0.borrow_mut() = Some(Ok(()));
                },
            }

            //唤醒等待异步创建目录的任务
            runtime.wakeup(task_id);

            //返回当前异步任务的默认值
            Default::default()
        };
        if let Err(e) = self.as_ref().runtime.spawn(task_id, task) {
            //派发异步创建目录的任务失败，则立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Create Dir Error, dir: {:?}, reason: {:?}", self.as_ref().path.as_ref(), e))));
        }

        //挂起当前任务，并返回值未就绪
        self.as_ref().runtime.pending(task_id, cx.waker().clone())
    }
}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> AsyncCreateDir<P, O> {
    //构建异步创建指定目录的方法
    pub fn new(runtime: MultiTaskRuntime<O>, path: P) -> Self {
        AsyncCreateDir {
            runtime,
            path,
            result: CreateDirResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

/*
* 异步创建目录
*/
pub async fn create_dir<P, O>(runtime: MultiTaskRuntime<O>, path: P) -> Result<()>
    where P: AsRef<Path> + Send + 'static, O: Default + 'static {
    AsyncCreateDir::new(runtime, path).await
}

/*
* 异步移除指定的目录的结果
*/
#[derive(Clone)]
struct RemoveDirResult(Arc<RefCell<Option<Result<()>>>>);

unsafe impl Send for RemoveDirResult {}
unsafe impl Sync for RemoveDirResult {}

/*
* 异步移除指定的目录
*/
struct AsyncRemoveDir<P: AsRef<Path> + Send + 'static, O: Default + 'static> {
    runtime:    MultiTaskRuntime<O>,    //异步运行时
    path:       P,                      //文件路径
    result:     RemoveDirResult,        //移除目录结果
}

unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Send for AsyncRemoveDir<P, O> {}
unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Sync for AsyncRemoveDir<P, O> {}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Future for AsyncRemoveDir<P, O> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.as_ref().result.0.borrow_mut().take() {
            //已移除指定目录，则返回
            return Poll::Ready(result);
        }

        //异步移除指定目录
        let task_id = self.as_ref().runtime.alloc();
        let runtime = self.as_ref().runtime.clone();
        let path = self.as_ref().path.as_ref().to_path_buf();
        let result = self.as_ref().result.clone();
        let task = async move {
            match sync_remove_dir(path) {
                Err(e) => {
                    //异步移除目录失败，则设置等待异步移除目录的任务的值
                    *result.0.borrow_mut() = Some(Err(e));
                },
                Ok(_) => {
                    //异步移除目录成功，则设置等待异步移除目录的任务的值
                    *result.0.borrow_mut() = Some(Ok(()));
                },
            }

            //唤醒等待异步移除目录的任务
            runtime.wakeup(task_id);

            //返回当前异步任务的默认值
            Default::default()
        };
        if let Err(e) = self.as_ref().runtime.spawn(task_id, task) {
            //派发异步移除目录的任务失败，则立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Remove Dir Error, dir: {:?}, reason: {:?}", self.as_ref().path.as_ref(), e))));
        }

        //挂起当前任务，并返回值未就绪
        self.as_ref().runtime.pending(task_id, cx.waker().clone())
    }
}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> AsyncRemoveDir<P, O> {
    //构建异步移除指定目录的方法
    pub fn new(runtime: MultiTaskRuntime<O>, path: P) -> Self {
        AsyncRemoveDir {
            runtime,
            path,
            result: RemoveDirResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

/*
* 异步移除目录
*/
pub async fn remove_dir<P, O>(runtime: MultiTaskRuntime<O>, path: P) -> Result<()>
    where P: AsRef<Path> + Send + 'static, O: Default + 'static {
    AsyncRemoveDir::new(runtime, path).await
}

/*
* 异步复制指定的文件的结果
*/
#[derive(Clone)]
struct CopyFileResult(Arc<RefCell<Option<Result<u64>>>>);

unsafe impl Send for CopyFileResult {}
unsafe impl Sync for CopyFileResult {}

/*
* 异步复制指定的文件
*/
struct AsyncCopyFile<P: AsRef<Path> + Send + 'static, O: Default + 'static> {
    runtime:    MultiTaskRuntime<O>,    //异步运行时
    from:       P,                      //源文件路径
    to:         P,                      //目录文件路径
    result:     CopyFileResult,         //复制文件结果
}

unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Send for AsyncCopyFile<P, O> {}
unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Sync for AsyncCopyFile<P, O> {}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Future for AsyncCopyFile<P, O> {
    type Output = Result<u64>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.as_ref().result.0.borrow_mut().take() {
            //已复制指定文件，则返回
            return Poll::Ready(result);
        }

        //异步复制指定文件
        let task_id = self.as_ref().runtime.alloc();
        let runtime = self.as_ref().runtime.clone();
        let from = self.as_ref().from.as_ref().to_path_buf();
        let to = self.as_ref().to.as_ref().to_path_buf();
        let result = self.as_ref().result.clone();
        let task = async move {
            match sync_copy(from, to) {
                Err(e) => {
                    //异步复制文件失败，则设置等待异步复制文件的任务的值
                    *result.0.borrow_mut() = Some(Err(e));
                },
                Ok(len) => {
                    //异步复制文件成功，则设置等待异步复制文件的任务的值
                    *result.0.borrow_mut() = Some(Ok((len)));
                },
            }

            //唤醒等待异步复制文件的任务
            runtime.wakeup(task_id);

            //返回当前异步任务的默认值
            Default::default()
        };
        if let Err(e) = self.as_ref().runtime.spawn(task_id, task) {
            //派发异步复制文件的任务失败，则立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Copy File Error, from: {:?}, to: {:?}, reason: {:?}", self.as_ref().from.as_ref(), self.as_ref().to.as_ref(), e))));
        }

        //挂起当前任务，并返回值未就绪
        self.as_ref().runtime.pending(task_id, cx.waker().clone())
    }
}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> AsyncCopyFile<P, O> {
    //构建异步复制指定文件的方法
    pub fn new(runtime: MultiTaskRuntime<O>, from: P, to: P) -> Self {
        AsyncCopyFile {
            runtime,
            from,
            to,
            result: CopyFileResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

/*
* 异步复制文件
*/
pub async fn copy_file<P, O>(runtime: MultiTaskRuntime<O>, from: P, to: P) -> Result<u64>
    where P: AsRef<Path> + Send + 'static, O: Default + 'static {
    AsyncCopyFile::new(runtime, from, to).await
}

/*
* 异步移除指定的文件的结果
*/
#[derive(Clone)]
struct RemoveFileResult(Arc<RefCell<Option<Result<()>>>>);

unsafe impl Send for RemoveFileResult {}
unsafe impl Sync for RemoveFileResult {}

/*
* 异步移作指定的文件
*/
struct AsyncRemoveFile<P: AsRef<Path> + Send + 'static, O: Default + 'static> {
    runtime:    MultiTaskRuntime<O>,    //异步运行时
    path:       P,                      //文件路径
    result:     RemoveFileResult,       //移除文件结果
}

unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Send for AsyncRemoveFile<P, O> {}
unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Sync for AsyncRemoveFile<P, O> {}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Future for AsyncRemoveFile<P, O> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.as_ref().result.0.borrow_mut().take() {
            //已移除指定文件，则返回
            return Poll::Ready(result);
        }

        //异步移除指定文件
        let task_id = self.as_ref().runtime.alloc();
        let runtime = self.as_ref().runtime.clone();
        let path = self.as_ref().path.as_ref().to_path_buf();
        let result = self.as_ref().result.clone();
        let task = async move {
            match sync_remove_file(path) {
                Err(e) => {
                    //异步移除文件失败，则设置等待异步移除文件的任务的值
                    *result.0.borrow_mut() = Some(Err(e));
                },
                Ok(_) => {
                    //异步移除文件成功，则设置等待异步移除文件的任务的值
                    *result.0.borrow_mut() = Some(Ok(()));
                },
            }

            //唤醒等待异步移除文件的任务
            runtime.wakeup(task_id);

            //返回当前异步任务的默认值
            Default::default()
        };
        if let Err(e) = self.as_ref().runtime.spawn(task_id, task) {
            //派发异步移除文件的任务失败，则立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Remove File Error, path: {:?}, reason: {:?}", self.as_ref().path.as_ref(), e))));
        }

        //挂起当前任务，并返回值未就绪
        self.as_ref().runtime.pending(task_id, cx.waker().clone())
    }
}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> AsyncRemoveFile<P, O> {
    //构建异步移除指定文件的方法
    pub fn new(runtime: MultiTaskRuntime<O>, path: P) -> Self {
        AsyncRemoveFile {
            runtime,
            path,
            result: RemoveFileResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

/*
* 异步移除文件
*/
pub async fn remove_file<P, O>(runtime: MultiTaskRuntime<O>, path: P) -> Result<()>
    where P: AsRef<Path> + Send + 'static, O: Default + 'static {
    AsyncRemoveFile::new(runtime, path).await
}

/*
* 文件选项
*/
pub enum AsyncFileOptions {
    OnlyRead,
    OnlyWrite,
    OnlyAppend,
    ReadAppend,
    ReadWrite,
    TruncateWrite,
}

/*
* 写文件选项
*/
#[derive(Debug, Clone)]
pub enum WriteOptions {
    None,
    Flush,
    Sync(bool),
    SyncAll(bool),
}

/*
* 异步内部文件
*/
struct InnerFile<O: Default + 'static> {
    runtime:        MultiTaskRuntime<O>,
    path:           PathBuf,
    inner:          RwLock<File>,
}

impl<O: Default + 'static> Debug for InnerFile<O> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "AsyncFile[file = {:?}]", self.path)
    }
}

/*
* 异步文件
*/
#[derive(Debug)]
pub struct AsyncFile<O: Default + 'static>(Arc<InnerFile<O>>);

unsafe impl<O: Default + 'static> Send for AsyncFile<O> {}
unsafe impl<O: Default + 'static> Sync for AsyncFile<O> {}

impl<O: Default + 'static> Clone for AsyncFile<O> {
    fn clone(&self) -> Self {
        AsyncFile(self.0.clone())
    }
}

/*
* 异步文件的同步方法
*/
impl<O: Default + 'static> AsyncFile<O> {
    //检查是否是符号链接
    pub fn is_symlink(&self) -> bool {
        self.0.inner.read().metadata().ok().unwrap().file_type().is_symlink()
    }

    //检查是否是文件
    pub fn is_file(&self) -> bool {
        self.0.inner.read().metadata().ok().unwrap().file_type().is_file()
    }

    //检查文件是否只读
    pub fn is_only_read(&self) -> bool {
        self.0.inner.read().metadata().ok().unwrap().permissions().readonly()
    }

    //获取文件大小
    pub fn get_size(&self) -> u64 {
        self.0.inner.read().metadata().ok().unwrap().len()
    }

    //获取文件修改时间
    pub fn get_modified_time(&self) -> Result<Duration> {
        match self.0.inner.read().metadata().ok().unwrap().modified() {
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
            Ok(time) => {
                match time.elapsed() {
                    Err(e) => Err(Error::new(ErrorKind::Other, e)),
                    Ok(duration) => Ok(duration),
                }
            },
        }
    }

    //获取文件访问时间
    pub fn get_accessed_time(&self) -> Result<Duration> {
        match self.0.inner.read().metadata().ok().unwrap().accessed() {
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
            Ok(time) => {
                match time.elapsed() {
                    Err(e) => Err(Error::new(ErrorKind::Other, e)),
                    Ok(duration) => Ok(duration),
                }
            },
        }
    }

    //获取文件创建时间
    pub fn get_created_time(&self) -> Result<Duration> {
        match self.0.inner.read().metadata().ok().unwrap().created() {
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
            Ok(time) => {
                match time.elapsed() {
                    Err(e) => Err(Error::new(ErrorKind::Other, e)),
                    Ok(duration) => Ok(duration),
                }
            },
        }
    }
}

/*
* 异步文件的异步方法
*/
impl<O: Default + 'static> AsyncFile<O> {
    //以指定方式异步打开指定的文件
    pub async fn open<P>(runtime: MultiTaskRuntime<O>,
                         path: P,
                         options: AsyncFileOptions) -> Result<Self>
        where P: AsRef<Path> + Send + 'static {
        AsyncOpenFile::new(runtime, path, options).await
    }

    //从指定位置开始异步读指定字节
    pub async fn read(&self, pos: u64, len: usize) -> Result<Vec<u8>> {
        if len == 0 {
            //无效的字节数，则立即返回
            return Ok(Vec::with_capacity(0));
        }

        let mut buf = Vec::with_capacity(len);
        unsafe { buf.set_len(len); }
        AsyncReadFile::new(self.0.runtime.clone(), buf, 0, self.clone(), pos, len, 0).await
    }

    //从指定位置开始异步写指定字节
    pub async fn write(&self, pos: u64, buf: Arc<Vec<u8>>, options: WriteOptions) -> Result<usize> {
        if buf.len() == 0 {
            //无效的字节数，则立即返回
            return Ok(0);
        }

        AsyncWriteFile::new(self.0.runtime.clone(), buf, 0, self.clone(), pos, options, 0).await
    }
}

/*
* 以指定方式异步打开指定文件的结果
*/
struct OpenFileResult<O: Default + 'static>(Arc<RefCell<Option<Result<AsyncFile<O>>>>>);

unsafe impl<O: Default + 'static> Send for OpenFileResult<O> {}
unsafe impl<O: Default + 'static> Sync for OpenFileResult<O> {}

impl<O: Default + 'static> Clone for OpenFileResult<O> {
    fn clone(&self) -> Self {
        OpenFileResult(self.0.clone())
    }
}

/*
* 以指定方式异步打开指定的文件
*/
struct AsyncOpenFile<P: AsRef<Path> + Send + 'static, O: Default + 'static> {
    runtime:    MultiTaskRuntime<O>,    //异步运行时
    path:       P,                      //文件路径
    options:    AsyncFileOptions,       //打开文件的选项
    result:     OpenFileResult<O>,      //打开文件结果
}

unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Send for AsyncOpenFile<P, O> {}
unsafe impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Sync for AsyncOpenFile<P, O> {}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> Future for AsyncOpenFile<P, O> {
    type Output = Result<AsyncFile<O>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.as_ref().result.0.borrow_mut().take() {
            //已打开指定文件，则返回
            return Poll::Ready(result);
        }

        //以指定方式异步打开指定文件
        let (r, w, a, c, t) = match self.as_ref().options {
            AsyncFileOptions::OnlyRead => (true, false, false, false, false),
            AsyncFileOptions::OnlyWrite => (false, true, false, true, false),
            AsyncFileOptions::OnlyAppend => (false, false, true, true, false),
            AsyncFileOptions::ReadAppend => (true, false, true, true, false),
            AsyncFileOptions::ReadWrite => (true, true, false, true, false),
            AsyncFileOptions::TruncateWrite => (false, true, false, true, true),
        };
        let task_id = self.as_ref().runtime.alloc();
        let runtime = self.as_ref().runtime.clone();
        let path = self.as_ref().path.as_ref().to_path_buf();
        let result = self.as_ref().result.clone();
        let task = async move {
            match OpenOptions::new()
                .read(r)
                .write(w)
                .append(a)
                .create(c)
                .truncate(t)
                .open(path.clone()) {
                Err(e) => {
                    //打开文件失败，则设置等待异步打开文件的任务的值
                    *result.0.borrow_mut() = Some(Err(e));
                },
                Ok(file) => {
                    //打开文件成功，则设置等待异步打开文件的任务的值
                    *result.0.borrow_mut() = Some(Ok(AsyncFile(Arc::new(InnerFile {
                        runtime: runtime.clone(),
                        path,
                        inner: RwLock::new(file),
                    }))));
                },
            }

            //唤醒等待异步打开文件的任务
            runtime.wakeup(task_id);

            //返回当前异步任务的默认值
            Default::default()
        };
        if let Err(e) = self.as_ref().runtime.spawn(task_id, task) {
            //派发异步打开文件的任务失败，则立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Open File Error, path: {:?}, reason: {:?}", self.as_ref().path.as_ref(), e))));
        }

        //挂起当前任务，并返回值未就绪
        self.as_ref().runtime.pending(task_id, cx.waker().clone())
    }
}

impl<P: AsRef<Path> + Send + 'static, O: Default + 'static> AsyncOpenFile<P, O> {
    //构建以指定方式异步打开指定文件的方法
    pub fn new(runtime: MultiTaskRuntime<O>, path: P, options: AsyncFileOptions) -> Self {
        AsyncOpenFile {
            runtime,
            path,
            options,
            result: OpenFileResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

/*
* 从指定位置开始异步读指定字节的结果
*/
#[derive(Clone)]
struct ReadFileResult(Arc<RefCell<Option<Result<Vec<u8>>>>>);

unsafe impl Send for ReadFileResult {}
unsafe impl Sync for ReadFileResult {}

/*
* 从指定位置开始异步读指定字节
*/
struct AsyncReadFile<O: Default + 'static> {
    runtime:    MultiTaskRuntime<O>,    //异步运行时
    buf:        Option<Vec<u8>>,        //读缓冲
    buf_pos:    u64,                    //读缓冲指针位置
    file:       AsyncFile<O>,           //文件
    pos:        u64,                    //文件指针位置
    len:        usize,                  //需要读取的字节数
    readed:     usize,                  //已读取的字节数
    result:     ReadFileResult,         //读取文件结果
}

unsafe impl<O: Default + 'static> Send for AsyncReadFile<O> {}
unsafe impl<O: Default + 'static> Sync for AsyncReadFile<O> {}

impl<O: Default + 'static> Future for AsyncReadFile<O> {
    type Output = Result<Vec<u8>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.as_ref().result.0.borrow_mut().take() {
            //已读取指定的字节，则返回
            return Poll::Ready(result);
        }

        //从指定位置开始异步读指定字节
        let task_id = self.as_ref().runtime.alloc();
        let runtime = self.as_ref().runtime.clone();
        let mut buf = self.as_mut().buf.take().unwrap();
        let buf_pos = self.as_ref().buf_pos;
        let file = self.as_ref().file.clone();
        let pos = self.as_ref().pos;
        let len = self.as_ref().len;
        let readed = self.as_ref().readed;
        let result = self.as_ref().result.clone();
        let task = async move {
            #[cfg(any(unix))]
                let r = file.0.inner.read().read_at(&mut buf[(buf_pos as usize)..(buf_pos as usize + len)], pos);
            #[cfg(any(windows))]
                let r = file.0.inner.read().seek_read(&mut buf[(buf_pos as usize)..(buf_pos as usize + len)], pos);

            match r {
                Ok(readed_len) if readed_len > 0 && readed_len < len => {
                    //读指定字节未完成，则继续读剩余字节
                    *result.0.borrow_mut() = Some(Self::new(runtime.clone(),
                                                            buf,
                                                            buf_pos + readed_len as u64,
                                                            file,
                                                            pos + readed_len as u64,
                                                            len - readed_len,
                                                            readed + readed_len).await);
                },
                Ok(readed_len) => {
                    //读指定字节完成，则设置等待异步读指定字节的任务的值
                    buf.truncate(readed + readed_len);
                    *result.0.borrow_mut() = Some(Ok(buf));
                },
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                    //需要尝试重读
                    *result.0.borrow_mut() = Some(Self::new(runtime.clone(), buf, buf_pos, file, pos, len, readed).await);
                },
                Err(e) => {
                    //读指定字节失败，则设置等待异步读指定字节的任务的值
                    *result.0.borrow_mut() = Some(Err(e));
                },
            }

            //唤醒等待异步读指定字节的任务
            runtime.wakeup(task_id);

            //返回当前异步任务的默认值
            Default::default()
        };
        if let Err(e) = self.as_ref().runtime.spawn(task_id, task) {
            //派发异步读指定字节的任务失败，则立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Read File Error, path: {:?}, reason: {:?}", self.as_ref().file.0.path, e))));
        }

        //挂起当前任务，并返回值未就绪
        self.as_ref().runtime.pending(task_id, cx.waker().clone())
    }
}

impl<O: Default + 'static> AsyncReadFile<O> {
    //构建从指定位置开始异步读指定字节的方法
    pub fn new(runtime: MultiTaskRuntime<O>, buf: Vec<u8>, buf_pos: u64, file: AsyncFile<O>, pos: u64, len: usize, readed: usize) -> Self {
        AsyncReadFile {
            runtime,
            buf: Some(buf),
            buf_pos,
            file,
            pos,
            len,
            readed,
            result: ReadFileResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

/*
* 从指定位置开始异步写指定字节的结果
*/
#[derive(Clone)]
struct WriteFileResult(Arc<RefCell<Option<Result<usize>>>>);

unsafe impl Send for WriteFileResult {}
unsafe impl Sync for WriteFileResult {}

/*
* 从指定位置开始异步读指定字节
*/
struct AsyncWriteFile<O: Default + 'static> {
    runtime:    MultiTaskRuntime<O>,    //异步运行时
    buf:        Arc<Vec<u8>>,           //写缓冲
    buf_pos:    u64,                    //写缓冲指针位置
    file:       AsyncFile<O>,           //文件
    pos:        u64,                    //文件指针位置
    options:    WriteOptions,           //写文件选项
    writed:     usize,                  //已写入的字节数
    result:     WriteFileResult,        //写入文件结果
}

unsafe impl<O: Default + 'static> Send for AsyncWriteFile<O> {}
unsafe impl<O: Default + 'static> Sync for AsyncWriteFile<O> {}

impl<O: Default + 'static> Future for AsyncWriteFile<O> {
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.as_ref().result.0.borrow_mut().take() {
            //已写入指定的字节，则返回
            return Poll::Ready(result);
        }

        //从指定位置开始异步读指定字节
        let task_id = self.as_ref().runtime.alloc();
        let runtime = self.as_ref().runtime.clone();
        let buf = self.as_mut().buf.clone();
        let buf_pos = self.as_ref().buf_pos;
        let file = self.as_ref().file.clone();
        let pos = self.as_ref().pos;
        let options = self.as_ref().options.clone();
        let writed = self.as_ref().writed;
        let result = self.as_ref().result.clone();
        let task = async move {
            #[cfg(any(unix))]
                let r = file.0.inner.read().write_at(&buf[(buf_pos as usize)..], pos);
            #[cfg(any(windows))]
                let r = file.0.inner.read().seek_write(&buf[(buf_pos as usize)..], pos);

            match r {
                Ok(writed_len) if writed_len < (buf.len() - buf_pos as usize) => {
                    //写指定字节未完成，则继续写剩余字节
                    *result.0.borrow_mut() = Some(Self::new(runtime.clone(),
                                                            buf,
                                                            buf_pos + writed_len as u64,
                                                            file,
                                                            pos + writed_len as u64,
                                                            options,
                                                            writed + writed_len).await);
                },
                Ok(writed_len) => {
                    //写指定字节完成，则根据写选项同步文件，并设置等待异步写指定字节的任务的值
                    let sync_result = match options {
                        WriteOptions::None => Ok(writed + writed_len),
                        WriteOptions::Flush => file.0.inner.write().flush().and(Ok(writed + writed_len)),
                        WriteOptions::Sync(true) => {
                            let flush_result = file.0.inner.write().flush();
                            flush_result
                                .and_then(|_| file.0.inner.read().sync_data())
                                .and(Ok(writed + writed_len))
                        },
                        WriteOptions::Sync(false) => file.0.inner.read().sync_data().and(Ok(writed + writed_len)),
                        WriteOptions::SyncAll(true) => {
                            let flush_result = file.0.inner.write().flush();
                            flush_result
                                .and_then(|_| file.0.inner.read().sync_all())
                                .and(Ok(writed + writed_len))
                        },
                        WriteOptions::SyncAll(false) => file.0.inner.read().sync_all().and(Ok(writed + writed_len)),
                    };
                    *result.0.borrow_mut() = Some(sync_result);
                },
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                    //需要尝试重写
                    *result.0.borrow_mut() = Some(Self::new(runtime.clone(), buf, buf_pos, file, pos, options, writed).await);
                },
                Err(e) => {
                    //写指定字节失败，则设置等待异步写指定字节的任务的值
                    *result.0.borrow_mut() = Some(Err(e));
                },
            }

            //唤醒等待异步写指定字节的任务
            runtime.wakeup(task_id);

            //返回当前异步任务的默认值
            Default::default()
        };
        if let Err(e) = self.as_ref().runtime.spawn(task_id, task) {
            //派发异步写指定字节的任务失败，则立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Write File Error, path: {:?}, reason: {:?}", self.as_ref().file.0.path, e))));
        }

        //挂起当前任务，并返回值未就绪
        self.as_ref().runtime.pending(task_id, cx.waker().clone())
    }
}

impl<O: Default + 'static> AsyncWriteFile<O> {
    //构建从指定位置开始异步读指定字节的方法
    pub fn new(runtime: MultiTaskRuntime<O>, buf: Arc<Vec<u8>>, buf_pos: u64, file: AsyncFile<O>, pos: u64, options: WriteOptions, writed: usize) -> Self {
        AsyncWriteFile {
            runtime,
            buf,
            buf_pos,
            file,
            pos,
            options,
            writed,
            result: WriteFileResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}