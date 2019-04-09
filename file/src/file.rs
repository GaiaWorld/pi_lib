use std::sync::Arc;
use std::path::Path;
use std::clone::Clone;
use std::boxed::FnBox;
use std::time::Duration;
#[cfg(any(unix))]
use std::os::unix::fs::FileExt;
#[cfg(any(windows))]
use std::os::windows::fs::FileExt;

use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::fs::{File, OpenOptions, Metadata, rename, remove_file};
use std::io::{Seek, Read, Write, Result, SeekFrom, Error, ErrorKind};

use atom::Atom;
use apm::counter::{GLOBAL_PREF_COLLECT, PrefCounter, PrefTimer};

use worker::task::TaskType;
use worker::impls::cast_store_task;

/*
* 文件块默认大小
*/
const BLOCK_SIZE: usize = 8192;

/*
* 文件异步访问任务类型
*/
const ASYNC_FILE_TASK_TYPE: TaskType = TaskType::Async(false);

/*
* 文件异步访问任务优先级
*/
const ASYNC_FILE_PRIORITY: usize = 100;

/*
* 打开异步文件信息
*/
const OPEN_ASYNC_FILE_INFO: &str = "open async file";

/*
* 读异步文件信息
*/
const READ_ASYNC_FILE_INFO: &str = "read async file";

/*
* 共享读异步文件信息
*/
const SHARED_READ_ASYNC_FILE_INFO: &str = "shared read async file";

/*
* 写异步文件信息
*/
const WRITE_ASYNC_FILE_INFO: &str = "write async file";

/*
* 共享写异步文件信息
*/
const SHARED_WRITE_ASYNC_FILE_INFO: &str = "shared write async file";

/*
* 重命名文件
*/
const RENAME_ASYNC_FILE_INFO: &str = "rename async file";

/*
* 移除文件信息
*/
const REMOVE_ASYNC_FILE_INFO: &str = "remove async file";

lazy_static! {
    //打开只读异步文件数量
    static ref ONLY_READ_ASYNC_FILE_OPEN_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("only_read_async_file_open_count"), 0).unwrap();
    //打开只写异步文件数量
    static ref ONLY_WRITE_ASYNC_FILE_OPEN_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("only_write_async_file_open_count"), 0).unwrap();
    //打开只追加异步文件数量
    static ref ONLY_APPEND_ASYNC_FILE_OPEN_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("only_append_async_file_open_count"), 0).unwrap();
    //打开读追加异步文件数量
    static ref READ_APPEND_ASYNC_FILE_OPEN_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("read_append_async_file_open_count"), 0).unwrap();
    //打开读写异步文件数量
    static ref READ_WRITE_ASYNC_FILE_OPEN_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("read_write_async_file_open_count"), 0).unwrap();
    //打开覆写异步文件数量
    static ref COVER_WRITE_ASYNC_FILE_OPEN_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("cover_write_async_file_open_count"), 0).unwrap();
    //读异步文件成功次数
    static ref READ_ASYNC_FILE_OK_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("read_async_file_ok_count"), 0).unwrap();
    //读异步文件失败次数
    static ref READ_ASYNC_FILE_ERROR_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("read_async_file_error_count"), 0).unwrap();
    //读异步文件字节数
    static ref READ_ASYNC_FILE_BYTE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("read_async_file_byte_count"), 0).unwrap();
    //写异步文件成功次数
    static ref WRITE_ASYNC_FILE_OK_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("write_async_file_ok_count"), 0).unwrap();
    //写异步文件失败次数
    static ref WRITE_ASYNC_FILE_ERROR_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("write_async_file_error_count"), 0).unwrap();
    //写异步文件字节数
    static ref WRITE_ASYNC_FILE_BYTE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("write_async_file_byte_count"), 0).unwrap();
}

/*
* 文件选项
*/
pub enum AsyncFileOptions {
    OnlyRead(u8),
    OnlyWrite(u8),
    OnlyAppend(u8),
    ReadAppend(u8),
    ReadWrite(u8),
    TruncateWrite(u8),
}

/*
* 写文件选项
*/
pub enum WriteOptions {
    None,
    Flush,
    Sync(bool),
    SyncAll(bool),
}

/*
* 共享接口
*/
pub trait Shared {
    type T;

    //通过异步文件构建共享异步文件
    fn new(file: Self::T) -> Arc<Self::T>;

    //原子的从指定位置开始读指定字节
    fn pread(self, pos: u64, len: usize, callback: Box<FnBox(Arc<Self::T>, Result<Vec<u8>>)>);

    //原子的从指定位置开始读指定字节，并填充指定的向量
    fn fpread(self, buf: Vec<u8>, buf_pos: u64, pos: u64, len: usize, callback: Box<FnBox(Arc<Self::T>, Result<Vec<u8>>)>);

    //原子的从指定位置开始写指定字节
    fn pwrite(self, options: WriteOptions, pos: u64, bytes: Vec<u8>, callback: Box<FnBox(Arc<Self::T>, Result<usize>)>);
}

/*
* 共享异步文件
*/
pub type SharedFile = Arc<AsyncFile>;

impl Shared for SharedFile {
    type T = AsyncFile;

    fn new(file: Self::T) -> Arc<Self::T> {
        Arc::new(file)
    }

    fn pread(self, pos: u64, len: usize, callback: Box<FnBox(Arc<Self::T>, Result<Vec<u8>>)>) {
        if len == 0 {
            READ_ASYNC_FILE_ERROR_COUNT.sum(1);

            return callback(self, Err(Error::new(ErrorKind::Other, "pread failed, invalid len")));
        }

        let mut vec: Vec<u8> = Vec::with_capacity(len);
        vec.resize(len, 0);
        pread_continue(vec, 0, self, pos, len, callback);
    }

    fn fpread(self, buf: Vec<u8>, buf_pos: u64, pos: u64, len: usize, callback: Box<FnBox(Arc<Self::T>, Result<Vec<u8>>)>) {
        if len == 0 {
            READ_ASYNC_FILE_ERROR_COUNT.sum(1);

            return callback(self, Err(Error::new(ErrorKind::Other, "fpread failed, invalid len")));
        }

        let buf_len = buf.len();
        let mut vec = buf;
        if (buf_len as isize - buf_pos as isize) < len as isize {
            //当前空间不够，则扩容并初始化
            if buf_pos as usize > buf_len {
                //偏移大于当前长度
                vec.resize(buf_pos as usize + len, 0);
            } else {
                //偏移小于等于当前长度
                vec.resize(buf_len as usize + len, 0);
            }
        }
        fpread_continue(vec, buf_pos, self, pos, len, callback);
    }

    fn pwrite(self, options: WriteOptions, pos: u64, bytes: Vec<u8>, callback: Box<FnBox(Arc<Self::T>, Result<usize>)>) {
        let len = bytes.len();
        if len == 0 {
            return callback(self, Ok(0));
        }

        pwrite_continue(len, self, options, pos, bytes, 0, callback);
    }
}

/*
* 异步文件
*/
pub struct AsyncFile{
    inner: File, 
    buffer_size: usize, 
    pos: u64, 
    buffer: Option<Vec<u8>>
}

impl Debug for AsyncFile {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "AsyncFile[file = {:?}, buffer_size = {}, current_pos = {}, buffer_len = {}, buffer_size = {}]", 
            self.inner, self.buffer_size, self.pos, self.buffer.as_ref().unwrap().len(), self.buffer.as_ref().unwrap().capacity())
    }
}

impl Clone for AsyncFile {
    fn clone(&self) -> Self {
        match self.inner.try_clone() {
            Err(e) => panic!("{:?}", e),
            Ok(inner) => {
                AsyncFile {
                    inner: inner, 
                    buffer_size: self.buffer_size, 
                    pos: 0, 
                    buffer: Some(Vec::with_capacity(0))
                }
            },
        }
    }
}

impl AsyncFile {
    //以指定方式打开指定文件
    pub fn open<P: AsRef<Path> + Send + 'static>(path: P, options: AsyncFileOptions, callback: Box<FnBox(Result<Self>)>) {
        let func = move |_lock| {
            let (r, w, a, c, t, len) = match options {
                AsyncFileOptions::OnlyRead(len) => (true, false, false, false, false, len),
                AsyncFileOptions::OnlyWrite(len) => (false, true, false, true, false, len),
                AsyncFileOptions::OnlyAppend(len) => (false, false, true, true, false, len),
                AsyncFileOptions::ReadAppend(len) => (true, false, true, true, false, len),
                AsyncFileOptions::ReadWrite(len) => (true, true, false, true, false, len),
                AsyncFileOptions::TruncateWrite(len) => (false, true, false, true, true, len),
            };

            match OpenOptions::new()
                            .read(r)
                            .write(w)
                            .append(a)
                            .create(c)
                            .truncate(t)
                            .open(path) {
                Err(e) => callback(Err(e)),
                Ok(file) => {
                    match options {
                        AsyncFileOptions::OnlyRead(_) => ONLY_READ_ASYNC_FILE_OPEN_COUNT.sum(1),
                        AsyncFileOptions::OnlyWrite(_) => ONLY_WRITE_ASYNC_FILE_OPEN_COUNT.sum(1),
                        AsyncFileOptions::OnlyAppend(_) => ONLY_APPEND_ASYNC_FILE_OPEN_COUNT.sum(1),
                        AsyncFileOptions::ReadAppend(_) => READ_APPEND_ASYNC_FILE_OPEN_COUNT.sum(1),
                        AsyncFileOptions::ReadWrite(_) => READ_WRITE_ASYNC_FILE_OPEN_COUNT.sum(1),
                        AsyncFileOptions::TruncateWrite(_) => COVER_WRITE_ASYNC_FILE_OPEN_COUNT.sum(1),
                    };

                    let buffer_size = match file.metadata() {
                        Ok(meta) => get_block_size(&meta) * len as usize,
                        _ => BLOCK_SIZE * len as usize,
                    };
                    callback(Ok(AsyncFile {
                                            inner: file, 
                                            buffer_size: buffer_size, 
                                            pos: 0, 
                                            buffer: Some(Vec::with_capacity(0))
                                        }))
                },
            }
        };
        cast_store_task(ASYNC_FILE_TASK_TYPE, ASYNC_FILE_PRIORITY, None, Box::new(func), Atom::from(OPEN_ASYNC_FILE_INFO));
    }

    //文件重命名
    pub fn rename<P: AsRef<Path> + Clone + Send + 'static>(from: P, to: P, callback: Box<FnBox(P, P, Result<()>)>) {
        let func = move |_lock| {
            let result = rename(from.clone(), to.clone());
            callback(from, to, result);
        };
        cast_store_task(ASYNC_FILE_TASK_TYPE, ASYNC_FILE_PRIORITY, None, Box::new(func), Atom::from(RENAME_ASYNC_FILE_INFO));
    }

    //移除指定文件
    pub fn remove<P: AsRef<Path> + Send + 'static>(path: P, callback: Box<FnBox(Result<()>)>) {
        let func = move |_lock| {
            let result = remove_file(path);
            callback(result);
        };
        cast_store_task(ASYNC_FILE_TASK_TYPE, ASYNC_FILE_PRIORITY, None, Box::new(func), Atom::from(REMOVE_ASYNC_FILE_INFO));
    }

    //检查是否是符号链接
    pub fn is_symlink(&self) -> bool {
        self.inner.metadata().ok().unwrap().file_type().is_symlink()
    }

    //检查是否是文件
    pub fn is_file(&self) -> bool {
        self.inner.metadata().ok().unwrap().file_type().is_file()
    }

    //检查文件是否只读
    pub fn is_only_read(&self) -> bool {
        self.inner.metadata().ok().unwrap().permissions().readonly()
    }
    
    //获取文件大小
    pub fn get_size(&self) -> u64 {
        self.inner.metadata().ok().unwrap().len()
    }

    //获取文件修改时间
    pub fn get_modified_time(&self) -> Option<Duration> {
        match self.inner.metadata().ok().unwrap().modified() {
            Ok(time) => {
                match time.elapsed() {
                    Ok(duration) => Some(duration),
                    _ => None,
                }
            },
            _ => None,
        }
    }

    //获取文件访问时间
    pub fn get_accessed_time(&self) -> Option<Duration> {
        match self.inner.metadata().ok().unwrap().accessed() {
            Ok(time) => {
                match time.elapsed() {
                    Ok(duration) => Some(duration),
                    _ => None,
                }
            },
            _ => None,
        }
    }

    //获取文件创建时间
    pub fn get_created_time(&self) -> Option<Duration> {
        match self.inner.metadata().ok().unwrap().created() {
            Ok(time) => {
                match time.elapsed() {
                    Ok(duration) => Some(duration),
                    _ => None,
                }
            },
            _ => None,
        }
    }

    //从指定位置开始，读指定字节
    pub fn read(mut self, pos: u64, len: usize, callback: Box<FnBox(Self, Result<Vec<u8>>)>) {
        let func = move |_lock| {
            let file_size = self.get_size();
            if file_size == 0 || len == 0 {
                READ_ASYNC_FILE_OK_COUNT.sum(1);

                let vec = self.buffer.take().unwrap();
                callback(init_read_file(self), Ok(vec));
                return;
            } else {
                self = alloc_buffer(self, file_size, len);
            }
            
            //保证在append时，当前位置也不会被改变
            match self.inner.seek(SeekFrom::Start(pos as u64)) {
                Err(e) => {
                    READ_ASYNC_FILE_ERROR_COUNT.sum(1);

                    callback(init_read_file(self), Err(e))
                },
                Ok(_) => {
                    let buf_cap = self.buffer.as_ref().unwrap().capacity() as isize;
                    match  buf_cap - self.pos as isize {
                        diff if diff > 0 => {
                            let buf_size = if diff as usize >= self.buffer_size {
                                self.buffer_size
                            } else {
                                diff as usize
                            };
                            
                            match self.inner.read(&mut self.buffer.as_mut().unwrap()[(self.pos as usize)..(self.pos as usize + buf_size)]) {
                                Ok(n) if n == 0 || n < buf_size => {
                                    //文件尾
                                    self.pos = self.buffer.as_ref().unwrap().len() as u64;
                                    let vec = self.buffer.take().unwrap();

                                    READ_ASYNC_FILE_BYTE_COUNT.sum(vec.len());
                                    READ_ASYNC_FILE_OK_COUNT.sum(1);

                                    callback(init_read_file(self), Ok(vec));
                                },
                                Ok(n) => {
                                    self.pos += n as u64;
                                    if self.pos >= buf_cap as u64 {
                                        //读完成
                                        let vec = self.buffer.take().unwrap();

                                        READ_ASYNC_FILE_BYTE_COUNT.sum(vec.len());
                                        READ_ASYNC_FILE_OK_COUNT.sum(1);

                                        callback(init_read_file(self), Ok(vec));
                                    } else {
                                        //继续读
                                        self.read(pos + n as u64, len - n, callback);
                                    }
                                },
                                Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                                    //重复读
                                    self.read(pos, len, callback);
                                },
                                Err(e) => {
                                    READ_ASYNC_FILE_ERROR_COUNT.sum(1);

                                    callback(init_read_file(self), Err(e))
                                },
                            }
                        },
                        _ => {
                            //读完成
                            let vec = self.buffer.take().unwrap();

                            READ_ASYNC_FILE_BYTE_COUNT.sum(vec.len());
                            READ_ASYNC_FILE_OK_COUNT.sum(1);

                            callback(init_read_file(self), Ok(vec));
                        },
                    }       
                },
            }
        };
        cast_store_task(ASYNC_FILE_TASK_TYPE, ASYNC_FILE_PRIORITY, None, Box::new(func), Atom::from(READ_ASYNC_FILE_INFO));
    }

    //从指定位置开始，写指定字节
    pub fn write(mut self, options: WriteOptions, pos: u64, bytes: Vec<u8>, callback: Box<FnBox(Self, Result<()>)>) {
        let func = move |_lock| {
            if !&bytes[self.pos as usize..].is_empty() {
                match self.inner.seek(SeekFrom::Start(pos as u64)) {
                    Err(e) => {
                        WRITE_ASYNC_FILE_ERROR_COUNT.sum(1);

                        callback(init_write_file(self), Err(e))
                    },
                    Ok(_) => {
                        match self.inner.write(&bytes[self.pos as usize..]) {
                            Ok(0) => {
                                WRITE_ASYNC_FILE_ERROR_COUNT.sum(1);

                                callback(init_write_file(self), Err(Error::new(ErrorKind::WriteZero, "write failed")))
                            },
                            Ok(n) => {
                                //继续写
                                WRITE_ASYNC_FILE_BYTE_COUNT.sum(n);

                                self.pos += n as u64;
                                self.write(options, pos + n as u64, bytes, callback);
                            },
                            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                                //重复写
                                self.write(options, pos, bytes, callback);
                            },
                            Err(e) => {
                                WRITE_ASYNC_FILE_ERROR_COUNT.sum(1);

                                callback(init_write_file(self), Err(e))
                            },
                        }
                    },
                }
            } else {
                //写完成
                WRITE_ASYNC_FILE_OK_COUNT.sum(1);

                let result = match options {
                    WriteOptions::None => Ok(()),
                    WriteOptions::Flush => self.inner.flush(),
                    WriteOptions::Sync(true) => self.inner.flush().and_then(|_| self.inner.sync_data()),
                    WriteOptions::Sync(false) => self.inner.sync_data(),
                    WriteOptions::SyncAll(true) => self.inner.flush().and_then(|_| self.inner.sync_all()),
                    WriteOptions::SyncAll(false) => self.inner.sync_all(),
                };
                callback(init_write_file(self), result);
            }
        };
        cast_store_task(ASYNC_FILE_TASK_TYPE, ASYNC_FILE_PRIORITY, None, Box::new(func), Atom::from(WRITE_ASYNC_FILE_INFO));
    }
}

#[inline]
fn init_read_file(mut file: AsyncFile) -> AsyncFile {
    file.pos = 0;
    file.buffer = Some(Vec::with_capacity(0));
    file
}

#[inline]
fn init_write_file(mut file: AsyncFile) -> AsyncFile {
    file.pos = 0;
    file
}

#[inline]
fn alloc_buffer(mut file: AsyncFile, file_size: u64, len: usize) -> AsyncFile {
    if file.buffer.as_ref().unwrap().len() == 0 {
        if file_size > len as u64 {
            file.buffer.as_mut().unwrap().reserve(len);
            file.buffer.as_mut().unwrap().resize(len, 0);
        } else {
            file.buffer.as_mut().unwrap().reserve(file_size as usize);
            file.buffer.as_mut().unwrap().resize(file_size as usize, 0);
        }
    }
    file
}

#[cfg(any(unix))]
fn get_block_size(meta: &Metadata) -> usize {
    use std::os::unix::fs::MetadataExt;
    meta.blksize() as usize
}

#[cfg(any(windows))]
fn get_block_size(_meta: &Metadata) -> usize {
    BLOCK_SIZE
}

//继续读
fn pread_continue(mut vec: Vec<u8>, vec_pos: u64, file: SharedFile, pos: u64, len: usize, callback: Box<FnBox(Arc<<SharedFile as Shared>::T>, Result<Vec<u8>>)>) {
    let func = move |_lock| {
        #[cfg(any(unix))]
        let r = file.inner.read_at(&mut vec[vec_pos as usize..(vec_pos as usize + len)], pos);
        #[cfg(any(windows))]
        let r = file.inner.seek_read(&mut vec[vec_pos as usize ..(vec_pos as usize + len)], pos);

        match r {
            Ok(0) => {
                //读完成
                READ_ASYNC_FILE_BYTE_COUNT.sum(vec.len());
                READ_ASYNC_FILE_OK_COUNT.sum(1);

                callback(file, Ok(vec));
            }
            Ok(short_len) if short_len < len => {
                //继续读
                pread_continue(vec, vec_pos, file, pos + short_len as u64, len - short_len, callback);
            },
            Ok(_len) => {
                //读完成
                READ_ASYNC_FILE_BYTE_COUNT.sum(vec.len());
                READ_ASYNC_FILE_OK_COUNT.sum(1);

                callback(file, Ok(vec));
            },
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                //重复读
                pread_continue(vec, vec_pos, file, pos, len, callback);
            },
            Err(e) => {
                READ_ASYNC_FILE_ERROR_COUNT.sum(1);

                callback(file, Err(e))
            },
        }
    };
    cast_store_task(ASYNC_FILE_TASK_TYPE, ASYNC_FILE_PRIORITY, None, Box::new(func), Atom::from(SHARED_READ_ASYNC_FILE_INFO));
}

//继续填充读
fn fpread_continue(mut vec: Vec<u8>, vec_pos: u64, file: SharedFile, pos: u64, len: usize, callback: Box<FnBox(Arc<<SharedFile as Shared>::T>, Result<Vec<u8>>)>) {
    let func = move |_lock| {
        #[cfg(any(unix))]
        let r = file.inner.read_at(&mut vec[vec_pos as usize..(vec_pos as usize + len)], pos);
        #[cfg(any(windows))]
        let r = file.inner.seek_read(&mut vec[vec_pos as usize..(vec_pos as usize + len)], pos);

        match r {
            Ok(0) => {
                //读完成
                READ_ASYNC_FILE_BYTE_COUNT.sum(vec.len());
                READ_ASYNC_FILE_OK_COUNT.sum(1);

                callback(file, Ok(vec));
            }
            Ok(short_len) if short_len < len => {
                //继续读
                fpread_continue(vec, vec_pos + short_len as u64, file, pos + short_len as u64, len - short_len, callback);
            },
            Ok(_len) => {
                //读完成
                READ_ASYNC_FILE_BYTE_COUNT.sum(vec.len());
                READ_ASYNC_FILE_OK_COUNT.sum(1);

                callback(file, Ok(vec));
            },
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                //重复读
                fpread_continue(vec, vec_pos, file, pos, len, callback);
            },
            Err(e) => {
                READ_ASYNC_FILE_ERROR_COUNT.sum(1);

                callback(file, Err(e))
            },
        }
    };
    cast_store_task(ASYNC_FILE_TASK_TYPE, ASYNC_FILE_PRIORITY, None, Box::new(func), Atom::from(SHARED_READ_ASYNC_FILE_INFO));
}

//继续写
fn pwrite_continue(len: usize, mut file: SharedFile, options: WriteOptions, pos: u64, bytes: Vec<u8>, vec_pos: u64, callback: Box<FnBox(Arc<<SharedFile as Shared>::T>, Result<usize>)>) {
    let func = move |_lock| {
        #[cfg(any(unix))]
        let r = file.inner.write_at(&bytes[vec_pos as usize..len], pos);
        #[cfg(any(windows))]
        let r = file.inner.seek_write(&bytes[vec_pos as usize..len], pos);

        match r {
            Ok(short_len) if short_len < len => {
                //继续写
                WRITE_ASYNC_FILE_BYTE_COUNT.sum(len);

                pwrite_continue(len - short_len, file, options, pos + short_len as u64, bytes, vec_pos + short_len as u64, callback);
            },
            Ok(len) => {
                WRITE_ASYNC_FILE_BYTE_COUNT.sum(len);
                WRITE_ASYNC_FILE_OK_COUNT.sum(1);

                //写完成
                let result = match options {
                    WriteOptions::None => Ok(len),
                    WriteOptions::Flush => Arc::make_mut(&mut file).inner.flush().and(Ok(len)),
                    WriteOptions::Sync(true) => Arc::make_mut(&mut file).inner.flush().and_then(|_| file.inner.sync_data()).and(Ok(len)),
                    WriteOptions::Sync(false) => Arc::make_mut(&mut file).inner.sync_data().and(Ok(len)),
                    WriteOptions::SyncAll(true) => Arc::make_mut(&mut file).inner.flush().and_then(|_| file.inner.sync_all()).and(Ok(len)),
                    WriteOptions::SyncAll(false) => Arc::make_mut(&mut file).inner.sync_all().and(Ok(len)),
                };
                callback(file, result);
            },
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                //重复写
                pwrite_continue(len, file, options, pos, bytes, vec_pos, callback);
            },
            Err(e) => {
                WRITE_ASYNC_FILE_ERROR_COUNT.sum(1);

                callback(file, Err(e))
            },
        }
    };
    cast_store_task(ASYNC_FILE_TASK_TYPE, ASYNC_FILE_PRIORITY, None, Box::new(func), Atom::from(SHARED_WRITE_ASYNC_FILE_INFO));
}