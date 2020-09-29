#[macro_use]
extern crate lazy_static;

use std::fs;
use std::env;
use std::thread;
use std::sync::Arc;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::future::Future;
use std::io::{Error, Result, ErrorKind};
use std::sync::atomic::{AtomicBool, Ordering};

use futures::future::{FutureExt, BoxFuture};
use num_cpus;

use r#async::{rt::{AsyncRuntime,
                   single_thread::{SingleTaskRunner, SingleTaskRuntime},
                   multi_thread::{MultiTaskPool, MultiTaskRuntime}}};
use async_file::file::{rename, AsyncFileOptions, AsyncFile};

mod frontend;
mod backend;
mod rust_backend;
mod ts_backend;
mod utils;

use frontend::parse_source;
use backend::{create_bind_crate, generate_crates_proxy_source};
use utils::{NATIVE_OBJECT_PROXY_FILE_DIR_NAME, check_crate, Crate, CrateInfo, ParseContext, ProxySourceGenerater, abs_path};
use futures::StreamExt;

/*
* 初始化异步运行时
*/
lazy_static! {
    static ref WORKER_RUNTIME: MultiTaskRuntime<()> = {
        let pool = MultiTaskPool::new("PI-JS-PROXY-GEN-WORKER-RT".to_string(), num_cpus::get_physical(), 8 * 1024 * 1024, 10, None);
        pool.startup(false)
    };
}

/*
* 递归分析指定库列表下的所有源文件，返回指定库列表中声明了导出的库列表
*/
pub async fn parse_crates(dirs: Vec<PathBuf>) -> Result<Vec<Crate>> {
    let mut crates = Vec::new();

    let mut map = WORKER_RUNTIME.map();
    for path in dirs {
        let future = async move {
            match parse_crate(path).await {
                Err(e) => {
                    Err(e)
                },
                Ok(c) => {
                    Ok(c)
                }
            }
        }.boxed();

        map.join(AsyncRuntime::Multi(WORKER_RUNTIME.clone()), future);
    }

    match map.map(AsyncRuntime::Multi(WORKER_RUNTIME.clone()), true).await {
        Err(e) => Err(e),
        Ok(vec) => {
            for r in vec {
                match r {
                    Err(e) => {
                        return Err(e);
                    },
                    Ok(c) => {
                        crates.push(c);
                    },
                }
            }

            Ok(crates)
        },
    }
}

/*
* 递归分析指定库下的所有源文件，递归调用异步函数，需要使用boxed的Future
*/
pub async fn parse_crate(path: PathBuf) -> Result<Crate> {
    if path.is_dir() {
        //是目录，则继续分析
        match check_crate(path.as_path()) {
            Err(e) => {
                Err(Error::new(ErrorKind::Other, format!("Parse crate failed, path: {:?}, reason: {:?}", path, e)))
            },
            Ok((crate_info, src_path)) => {
                match parse_source_dir(src_path).await {
                    Err(e) => {
                        Err(Error::new(ErrorKind::Other, format!("Parse crate failed, path: {:?}, reason: {:?}", path, e)))
                    },
                    Ok(source) => {
                        Ok(Crate::new(crate_info, source))
                    },
                }
            },
        }
    } else {
        Err(Error::new(ErrorKind::Other, format!("Parse crate failed, path: {:?}, reason: invalid dir", path)))
    }
}

/*
* 分析源码目录
*/
pub fn parse_source_dir(path: PathBuf) -> BoxFuture<'static, Result<Vec<ParseContext>>> {
    async move {
        match fs::read_dir(path.clone()) {
            Err(e) => {
                //读目录失败，则立即返回错误
                Err(Error::new(ErrorKind::Other, format!("Parse crate failed, path: {:?}, reason: {:?}", path, e)))
            },
            Ok(mut dir) => {
                let mut vec = Vec::new();

                while let Some(entry) = dir.next() {
                    if let Ok(e) = entry {
                        let child_path = e.path();
                        if child_path.is_dir() {
                            //子目录
                            match parse_source_dir(child_path).await {
                                Err(e) => {
                                    //分析子目录失败，则立即返回错误
                                    return Err(e);
                                },
                                Ok(child_vec) => {
                                    //分析子目录成功，则记录分析的子目录上下文列表，并继续
                                    for context in child_vec {
                                        vec.push(context);
                                    }
                                    continue;
                                },
                            }
                        } else if child_path.is_file() {
                            //文件
                            match AsyncFile::open(WORKER_RUNTIME.clone(), child_path.clone(), AsyncFileOptions::OnlyRead).await {
                                Err(e) => {
                                    //打开文件失败，则立即返回错误
                                    return Err(Error::new(ErrorKind::Other, format!("Parse crate failed, file: {:?}, reason: {:?}", child_path, e)));
                                },
                                Ok(file) => {
                                    //打开文件成功，则继续分析源码
                                    match file.read(0, file.get_size() as usize).await {
                                        Err(e) => {
                                            //读文件失败，则立即返回错误
                                            return Err(Error::new(ErrorKind::Other, format!("Parse crate failed, file: {:?}, reason: {:?}", child_path, e)));
                                        },
                                        Ok(bin) => {
                                            let mut context = ParseContext::new(child_path.as_path());
                                            match String::from_utf8(bin) {
                                                Err(e) => {
                                                    //将源码转换为UTF8字符串失败，则立即返回错误
                                                    return Err(Error::new(ErrorKind::Other, format!("Parse crate failed, file: {:?}, reason: {:?}", child_path, e)));
                                                },
                                                Ok(source) => {
                                                    if let Err(e) = parse_source(&mut context, source.as_str()) {
                                                        //分析源码失败，则立即返回错误
                                                        return Err(e);
                                                    }

                                                    //分析源码成功，则记录当前上下文
                                                    vec.push(context);
                                                },
                                            }
                                        },
                                    }
                                },
                            }
                        }
                    }
                }

                Ok(vec)
            },
        }
    }.boxed()
}

/*
* 分析声明了导出的库列表，则创建指定路径的代理库，并生成相应的代理文件和代理代码
*/
pub async fn generate_proxy_crate(path: PathBuf,
                                  ts_proxy_root: PathBuf,
                                  version: &str,
                                  edition: &str,
                                  crates: Vec<Crate>) -> Result<()> {
    match abs_path(path.as_path()) {
        Err(e) => {
            Err(Error::new(ErrorKind::Other, format!("Generate proxy crate failed, crate path: {:?}, reason: {:?}", path, e)))
        },
        Ok(proxy_crate_path) => {
            match abs_path(ts_proxy_root.as_path()) {
                Err(e) => {
                    Err(Error::new(ErrorKind::Other, format!("Generate proxy crate failed, ts path: {:?}, reason: {:?}", ts_proxy_root, e)))
                },
                Ok(mut proxy_ts_path) => {
                    proxy_ts_path = proxy_ts_path.join(NATIVE_OBJECT_PROXY_FILE_DIR_NAME); //实际的ts代理文件根路径

                    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                        Err(e) => {
                            //获取当前系统时间失败，则立即返回错误
                            return Err(Error::new(ErrorKind::Other, format!("Generate proxy crate failed, ts path: {:?}, reason: {:?}", proxy_crate_path, e)));
                        },
                        Ok(now) => {
                            //获取当前系统时间成功，重命名已存在的ts代理文件根目录
                            let proxy_ts_path_rename = PathBuf::from(proxy_ts_path.to_str().unwrap().to_string() + "_" + now.as_millis().to_string().as_str());

                            if let Err(e) = rename(WORKER_RUNTIME.clone(), proxy_ts_path.clone(), proxy_ts_path_rename.clone()).await {
                                if e.kind() != ErrorKind::NotFound {
                                    //重命名错误不是没找到ts代理文件根目录的错误，则立即返回错误
                                    return Err(Error::new(ErrorKind::Other, format!("Generate proxy crate failed, old ts path: {:?}, ts path: {:?}, reason: {:?}", proxy_ts_path_rename, proxy_ts_path, e)));
                                }
                            }
                        },
                    }

                    let parent_path = if let Some(p) = proxy_crate_path.parent() {
                        p.to_path_buf()
                    } else {
                        return Err(Error::new(ErrorKind::Other, format!("Generate proxy crate failed, path: {:?}, reason: not allowed for proxy crate with root path", path)));
                    };

                    match create_bind_crate(proxy_crate_path, parent_path, version, edition, crates.as_slice()).await {
                        Err(e) => {
                            Err(Error::new(ErrorKind::Other, format!("Generate proxy crate failed, path: {:?}, reason: {:?}", path, e)))
                        },
                        Ok(src_path) => {
                            let generater = ProxySourceGenerater::new();
                            if let Err(e) = generate_crates_proxy_source(&generater,
                                                                         crates,
                                                                         src_path.clone(),
                                                                         proxy_ts_path.clone()).await {
                                Err(Error::new(ErrorKind::Other, format!("Generate proxy crate failed, path: {:?}, reason: {:?}", path, e)))
                            } else {
                                Ok(())
                            }
                        },
                    }
                },
            }
        },
    }
}

/*
* 派发异步任务到代理生成器的异步运行时中运行
*/
pub fn spawn(task: impl Future<Output = ()> + Send + 'static) {
    WORKER_RUNTIME.spawn(WORKER_RUNTIME.alloc(), async move {
        task.await;
    });
}

#[test]
fn test_front_end() {
    use std::fs;
    use std::env;
    use std::path::PathBuf;

    let cwd = env::current_dir().unwrap();
    let filename = PathBuf::from(r#".\tests\src\test\_10.rs"#);
    let path = cwd.join(filename.strip_prefix("./").unwrap());
    if let Ok(source) = fs::read_to_string(&path) {
        let mut context = utils::ParseContext::new(path.as_path());
        if let Err(e) = frontend::parse_source(&mut context, source.as_str()) {
            panic!("Test front end failed, reason: {:?}", e);
        }

        println!("{:#?}", context);
    }
}

#[test]
fn test_create_bind_crate() {
    use std::fs;
    use std::env;
    use std::path::PathBuf;

    WORKER_RUNTIME.spawn(WORKER_RUNTIME.alloc(), async move {
        let cwd = env::current_dir().unwrap();
        let filename = PathBuf::from(r#".\tests\pi_v8_ext\"#);
        let path = cwd.join(filename.strip_prefix("./").unwrap());
        let root = PathBuf::from(r#".\tests"#);
        let crates = parse_crates(vec![PathBuf::from(r#".\tests\export_crate"#)]).await.unwrap();
        let ts_proxy_root = PathBuf::from(r#".\tests\pi_v8_ext\ts"#);

        match create_bind_crate(path, root, "0.1.0", "2018", crates.as_slice()).await {
            Err(e) => {
                panic!("Test create bind crate failed, {:?}", e);
            },
            Ok(src_path) => {
                let generater = ProxySourceGenerater::new();
                if let Err(e) = generate_crates_proxy_source(&generater, crates, src_path.clone(), ts_proxy_root.clone()).await {
                    panic!("Test generate proxy file failed, {:?}", e);
                }
            },
        }
    });

    thread::sleep(Duration::from_millis(1000000000));
}