use std::thread;
use std::sync::Arc;
use std::io::{ErrorKind, Write};
use std::time::Duration;

use r#async::rt::multi_thread::{MultiTaskPool, MultiTaskRuntime};
use async_file::file::{create_dir, rename, remove_file, remove_dir, AsyncFileOptions, WriteOptions, AsyncFile};

#[test]
fn test_async_file() {
    //初始化异步运行时
    let pool = MultiTaskPool::new("TestAsyncRuntime".to_string(), 8, 1024 * 1024, 10, None);
    let rt = pool.startup(true);

    let rt_copy = rt.clone();
    let vec = Vec::from("Hello 什么是 Async File 异步文件?");
    let buf = Arc::from(&vec[..]);
    let future = async move {
        if let Err(e) = remove_dir(rt_copy.clone(), "./test_async_file/test/".to_string()).await {
            if e.kind() != ErrorKind::NotFound {
                panic!("remove dir failed, dir: {:?}, reason: {:?}", "./test_async_file/test/", e);
            }
        }

        let mut test_dir = "./test_async_file/tmp/".to_string();
        if let Err(e) = create_dir(rt_copy.clone(), test_dir.clone()).await {
            panic!("create dir failed, dir: {:?}, reason: {:?}", &test_dir, e);
        }

        let from_dir = test_dir.clone();
        test_dir = "./test_async_file/test/".to_string();
        if let Err(e) = rename(rt_copy.clone(), from_dir, test_dir.clone()).await {
            panic!("rename dir failed, dir: {:?}, reason: {:?}", &test_dir, e);
        }

        let mut test_file = test_dir.to_string() + "/tmp.txt";
        match AsyncFile::open(rt_copy.clone(), test_file.clone(), AsyncFileOptions::ReadWrite).await {
            Err(e) => {
                panic!("open file failed, file: {:?}, reason: {:?}", &test_file, e);
            },
            Ok(file) => {
                if !file.is_file() || file.is_only_read() || file.is_symlink() {
                    panic!("invalid file, reason: invalid file meta");
                }

                if let Err(e) = file.write(0, buf, WriteOptions::SyncAll(true)).await {
                    panic!("write file failed, file: {:?}, reason: {:?}", &test_file, e);
                }

                let from_file = test_file.clone();
                test_file = test_dir.to_string() + "/test.txt";
                if let Err(e) = rename(rt_copy.clone(), from_file, test_file.clone()).await {
                    panic!("rename file failed, file: {:?}, reason: {:?}", &test_file, e);
                }

                match file.read(0, 1000).await {
                    Err(e) => {
                        panic!("read file failed, file: {:?}, reason: {:?}", &test_file, e);
                    },
                    Ok(bin) => {
                        assert_eq!("Hello 什么是 Async File 异步文件?".to_string(), unsafe { String::from_utf8_unchecked(bin) });
                    },
                }

                if let Err(e) = remove_file(rt_copy.clone(), test_file.clone()).await {
                    panic!("remove file failed, file: {:?}, reason: {:?}", &test_file, e);
                }
            },
        }
    };
    if let Err(e) = rt.spawn(rt.alloc(), future) {
        panic!("spawn test file task failed, reason: {:?}", e);
    }

    thread::sleep(Duration::from_millis(10000));
}

#[test]
fn test_async_file_truncate_read_write() {
    //初始化异步运行时
    let pool = MultiTaskPool::new("TestAsyncRuntime".to_string(), 8, 1024 * 1024, 10, None);
    let rt = pool.startup(true);

    let rt_copy = rt.clone();
    let vec = Vec::from("Hello 什么是 Async File 异步文件?");
    let buf: Arc<[u8]> = Arc::from(&vec[..]);
    let future = async move {
        if let Err(e) = remove_dir(rt_copy.clone(), "./test_async_file/test/".to_string()).await {
            if e.kind() != ErrorKind::NotFound {
                panic!("remove dir failed, dir: {:?}, reason: {:?}", "./test_async_file/test/", e);
            }
        }

        let mut test_dir = "./test_async_file/tmp/".to_string();
        if let Err(e) = create_dir(rt_copy.clone(), test_dir.clone()).await {
            panic!("create dir failed, dir: {:?}, reason: {:?}", &test_dir, e);
        }

        let from_dir = test_dir.clone();
        test_dir = "./test_async_file/test/".to_string();
        if let Err(e) = rename(rt_copy.clone(), from_dir, test_dir.clone()).await {
            panic!("rename dir failed, dir: {:?}, reason: {:?}", &test_dir, e);
        }

        let mut test_file = test_dir.to_string() + "/tmp.txt";
        match AsyncFile::open(rt_copy.clone(), test_file.clone(), AsyncFileOptions::TruncateReadWrite).await {
            Err(e) => {
                panic!("open file failed, file: {:?}, reason: {:?}", &test_file, e);
            },
            Ok(file) => {
                if !file.is_file() || file.is_only_read() || file.is_symlink() {
                    panic!("invalid file, reason: invalid file meta");
                }

                if let Err(e) = file.write(0, buf, WriteOptions::SyncAll(true)).await {
                    panic!("write file failed, file: {:?}, reason: {:?}", &test_file, e);
                }

                let vec1 = Vec::from("Hello");
                let buf1: Arc<[u8]> = Arc::from(&vec1[..]);
                if let Err(e) = file.write(0, buf1, WriteOptions::SyncAll(true)).await {
                    panic!("write file failed, file: {:?}, reason: {:?}", &test_file, e);
                }

                let from_file = test_file.clone();
                test_file = test_dir.to_string() + "/test.txt";
                if let Err(e) = rename(rt_copy.clone(), from_file, test_file.clone()).await {
                    panic!("rename file failed, file: {:?}, reason: {:?}", &test_file, e);
                }

                match file.read(0, 1000).await {
                    Err(e) => {
                        panic!("read file failed, file: {:?}, reason: {:?}", &test_file, e);
                    },
                    Ok(bin) => {
                        assert_eq!("Hello".to_string(), unsafe { String::from_utf8_unchecked(bin) });
                    },
                }

                if let Err(e) = remove_file(rt_copy.clone(), test_file.clone()).await {
                    panic!("remove file failed, file: {:?}, reason: {:?}", &test_file, e);
                }
            },
        }
    };
    if let Err(e) = rt.spawn(rt.alloc(), future) {
        panic!("spawn test file task failed, reason: {:?}", e);
    }

    thread::sleep(Duration::from_millis(10000));
}