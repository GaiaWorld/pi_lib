//! # 用于指定的Rust库中满足导出规定的Rust代码的分析，并生成用于pi_v8的中间Rust代码和Typescript脚本
//!
//! * 整个过程分为两部分：
//!     - Rust代码分析并生成语法树，也就是前端处理
//!     - 解析语法树并生成中间Rust代码和Typescript脚本，也就是后端处理
//!
#![feature(pattern)]
#![feature(path_file_prefix)]

#[macro_use]
extern crate lazy_static;
extern crate core;

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use std::future::Future;
use std::io::{Error, Result, ErrorKind};

use futures::future::{FutureExt, BoxFuture};
use num_cpus;
use normpath::PathExt;

use pi_async::rt::{AsyncRuntime,
                   multi_thread::{MultiTaskRuntimeBuilder, StealableTaskPool, MultiTaskRuntime}};
use pi_async_file::file::{rename, AsyncFileOptions, AsyncFile};

mod frontend;
mod backend;
mod rust_backend;
mod ts_backend;
mod utils;

use frontend::parse_source;
use backend::{create_bind_crate, generate_crates_proxy_source};
use utils::{RUST_SOURCE_FILE_EXTENSION,
            NATIVE_OBJECT_PROXY_FILE_DIR_NAME,
            check_crate,
            Crate,
            MacroExpandPathBuf,
            MacroExpander,
            ParseContext,
            ProxySourceGenerater,
            abs_path};

///
/// 默认的宏展开文件后缀
///
const DEAFULT_MACRO_EXPAND_FILE_SUFFIX: &str = "__$expand$__";

///
/// 初始化异步运行时
///
lazy_static! {
    static ref WORKER_RUNTIME: MultiTaskRuntime<()> = {
        let len = num_cpus::get_physical();
        let pool = StealableTaskPool::with(len, len);
        let builder = MultiTaskRuntimeBuilder::new(pool)
            .thread_prefix("PI-JS-PROXY-GEN-WORKER-RT")
            .thread_stack_size(8 * 1024 * 1024)
            .init_worker_size(len)
            .set_worker_limit(len, len)
            .set_timeout(10);
        builder.build()
    };
}

///
/// 递归分析指定库列表下的所有源文件，可以指定是否展开宏或并发分析，返回指定库列表中声明了导出的库列表
///
pub async fn parse_crates(dirs: Vec<PathBuf>,
                          requrie_extand_macro_filenames: Option<Vec<String>>,
                          is_concurrent: bool) -> Result<Vec<Crate>> {
    let mut crates = Vec::new();

    if is_concurrent {
        //并发递归分析，导出函数的序号不保证一致
        let mut map = WORKER_RUNTIME.map_reduce(dirs.len());
        for path in dirs {
            let ignores_copy = requrie_extand_macro_filenames.clone();
            let future = async move {
                match parse_crate(path, ignores_copy).await {
                    Err(e) => {
                        Err(e)
                    },
                    Ok(c) => {
                        Ok(c)
                    }
                }
            }.boxed();

            map.map(WORKER_RUNTIME.clone(), future);
        }

        match map.reduce(true).await {
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
    } else {
        //顺序递归分析，导出函数的序号保证一致
        for path in dirs {
            match parse_crate(path, requrie_extand_macro_filenames.clone()).await {
                Err(e) => {
                    return Err(e);
                },
                Ok(c) => {
                    crates.push(c);
                }
            }
        }

        Ok(crates)
    }
}

///
/// 递归分析指定库下的所有源文件，可以指定是否展开宏
/// 递归调用异步函数，需要使用boxed的Future
///
pub async fn parse_crate(path: PathBuf,
                         requrie_extand_macro_filenames: Option<Vec<String>>) -> Result<Crate> {
    if path.is_dir() {
        //是目录，则继续分析
        match check_crate(path.as_path()) {
            Err(e) => {
                Err(Error::new(ErrorKind::Other, format!("Parse crate failed, path: {:?}, reason: {:?}", path, e)))
            },
            Ok((crate_info, src_path)) => {
                let macro_expander = if let Some(requires) = requrie_extand_macro_filenames {
                    //需要宏展开指定库下的所有源文件，则构建一个宏展开器
                    Some(MacroExpander::new(&path,
                                            &src_path,
                                            DEAFULT_MACRO_EXPAND_FILE_SUFFIX,
                                            requires))
                } else {
                    None
                };

                match parse_source_dir(src_path, macro_expander).await {
                    Err(e) => {
                        Err(Error::new(ErrorKind::Other, format!("Parse crate failed, path: {:?}, reason: {:?}", path, e)))
                    },
                    Ok(source) => {
                        match path.normalize() {
                            Err(e) => {
                                //获取指定库的本地绝对路径失败
                                Err(Error::new(ErrorKind::Other, format!("Parse crate path failed, path: {:?}, reason: {:?}", path, e)))
                            },
                            Ok(p) => {
                                //获取指定库的本地绝对路径成功
                                let crate_path = p.as_path().to_path_buf();
                                Ok(Crate::new(crate_path, crate_info, source))
                            },
                        }
                    },
                }
            },
        }
    } else {
        Err(Error::new(ErrorKind::Other, format!("Parse crate failed, path: {:?}, reason: invalid dir", path)))
    }
}

///
/// 分析源码目录
///
pub fn parse_source_dir(path: PathBuf,
                        macro_expander: Option<MacroExpander>) -> BoxFuture<'static, Result<Vec<ParseContext>>> {
    async move {
        match fs::read_dir(path.clone()) {
            Err(e) => {
                //读目录失败，则立即返回错误
                Err(Error::new(ErrorKind::Other, format!("Parse crate failed, path: {:?}, reason: {:?}", path, e)))
            },
            Ok(mut dir) => {
                let mut child_paths = Vec::new();
                while let Some(entry) = dir.next() {
                    if let Ok(e) = entry {
                        let child_path = e.path();
                        child_paths.push(child_path);
                    }
                }
                child_paths.sort(); //对当前目录下所有的同级路径进行排序

                let mut vec = Vec::new();
                for child_path in child_paths {
                    if child_path.is_dir() {
                        //子目录
                        match parse_source_dir(child_path, macro_expander.clone()).await {
                            Err(e) => {
                                //分析子目录失败，则立即返回错误
                                return Err(e);
                            },
                            Ok(child_vec) => {
                                //分析子目录成功，则记录分析的子目录上下文列表，并继续分析下一个文件
                                for context in child_vec {
                                    vec.push(context);
                                }
                                continue;
                            },
                        }
                    } else if child_path.is_file() {
                        //文件
                        if let Some(child_path_str) = child_path.to_str() {
                            if let Some(_) = child_path_str.find(DEAFULT_MACRO_EXPAND_FILE_SUFFIX) {
                                //如果文件是临时生成的宏展开后的源码文件，则忽略，并继续分析下一个文件
                                continue;
                            }
                        }

                        if let Some(ext) = child_path.extension() {
                            //有扩展名
                            if let Some(ext_str) = ext.to_str() {
                                if ext_str != RUST_SOURCE_FILE_EXTENSION {
                                    //不是Rust源码文件，则忽略，并继续分析下一个文件
                                    continue;
                                }

                                if child_path.metadata()?.len() == 0 {
                                    //当前源文件大小为0，则忽略，并继续分析下一个文件
                                    continue;
                                }
                            } else {
                                //无效扩展名，则忽略，并继续分析下一个文件
                                continue;
                            }
                        } else {
                            //无扩展名，则忽略，并继续分析下一个文件
                            continue;
                        }

                        let real_child_path = if let Some(expander) = &macro_expander {
                            //当前源文件需要宏展开
                            match expander.expand(&child_path) {
                                Err(e) => {
                                    //宏展开失败，则立即返回错误
                                    return Err(Error::new(ErrorKind::Other, format!("Parse crate failed, file: {:?}, reason: {:?}", child_path, e)));
                                },
                                Ok(None) => {
                                    //当前源文件不需要宏展开
                                    MacroExpandPathBuf::new(child_path.clone(), false)
                                },
                                Ok(Some(path)) => {
                                    //宏展开成功
                                    path
                                },
                            }
                        } else {
                            //当前源文件不需要宏展开
                            MacroExpandPathBuf::new(child_path.clone(), false)
                        };

                        match AsyncFile::open(WORKER_RUNTIME.clone(),
                                              real_child_path.clone(),
                                              AsyncFileOptions::OnlyRead).await {
                            Err(e) => {
                                //打开文件失败，则立即返回错误
                                return Err(Error::new(ErrorKind::Other, format!("Parse crate failed, file: {:?}, reason: {:?}", real_child_path.as_ref(), e)));
                            },
                            Ok(file) => {
                                //打开文件成功，则继续分析源码
                                match file.read(0, file.get_size() as usize).await {
                                    Err(e) => {
                                        //读文件失败，则立即返回错误
                                        return Err(Error::new(ErrorKind::Other, format!("Parse crate failed, file: {:?}, reason: {:?}", real_child_path.as_ref(), e)));
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

                Ok(vec)
            },
        }
    }.boxed()
}

///
/// 分析声明了导出的库列表，则创建指定路径的代理库，并生成相应的代理文件和代理代码，可以指定是否并发生成
///
pub async fn generate_proxy_crate(path: PathBuf,
                                  ts_proxy_root: PathBuf,
                                  vm_builtin_path: PathBuf,
                                  version: &str,
                                  edition: &str,
                                  is_concurrent: bool,
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
                            //获取当前系统时间成功，则重命名已存在的ts代理文件根目录
                            let proxy_ts_path_rename = PathBuf::from(proxy_ts_path.to_str().unwrap().to_string() + "_" + now.as_millis().to_string().as_str());

                            if let Err(e) = rename(WORKER_RUNTIME.clone(), proxy_ts_path.clone(), proxy_ts_path_rename.clone()).await {
                                if e.kind() != ErrorKind::NotFound {
                                    //重命名错误不是没找到ts代理文件根目录的错误，则立即返回错误
                                    return Err(Error::new(ErrorKind::Other, format!("Generate proxy crate failed, old ts path: {:?}, ts path: {:?}, reason: {:?}", proxy_ts_path_rename, proxy_ts_path, e)));
                                }
                            }
                        },
                    }

                    match create_bind_crate(proxy_crate_path,
                                            vm_builtin_path,
                                            version,
                                            edition,
                                            crates.as_slice()).await {
                        Err(e) => {
                            Err(Error::new(ErrorKind::Other, format!("Generate proxy crate failed, path: {:?}, reason: {:?}", path, e)))
                        },
                        Ok(src_path) => {
                            let generater = ProxySourceGenerater::new();
                            if let Err(e) = generate_crates_proxy_source(&generater,
                                                                         crates,
                                                                         src_path.clone(),
                                                                         proxy_ts_path.clone(),
                                                                         is_concurrent).await {
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

///
/// 派发异步任务到代理生成器的异步运行时中运行
///
pub fn spawn(task: impl Future<Output = ()> + Send + 'static) {
    WORKER_RUNTIME.spawn(WORKER_RUNTIME.alloc(), async move {
        task.await;
    }).unwrap();
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
    use std::env;
    use std::path::PathBuf;
    use std::time::Duration;

    WORKER_RUNTIME.spawn(WORKER_RUNTIME.alloc(), async move {
        let cwd = env::current_dir().unwrap();
        let filename = PathBuf::from(r#".\tests\pi_v8_ext\"#);
        let path = cwd.join(filename.strip_prefix("./").unwrap());
        let vm_builtin_path = PathBuf::from(r#"..\..\pi_v8\vm_builtin"#);
        let crates = parse_crates(vec![PathBuf::from(r#".\tests\export_crate"#)],
                                  None,
                                  true).await.unwrap();
        let ts_proxy_root = PathBuf::from(r#".\tests\pi_v8_ext\ts"#);

        match create_bind_crate(path,
                                vm_builtin_path,
                                "0.1.0",
                                "2018",
                                crates.as_slice()).await {
            Err(e) => {
                panic!("Test create bind crate failed, {:?}", e);
            },
            Ok(src_path) => {
                let generater = ProxySourceGenerater::new();
                if let Err(e) = generate_crates_proxy_source(&generater, crates, src_path.clone(), ts_proxy_root.clone(), false).await {
                    panic!("Test generate proxy file failed, {:?}", e);
                }
            },
        }
    }).unwrap();

    std::thread::sleep(Duration::from_millis(1000000000));
}

//测试在导出库中使用宏来生成导出函数
#[test]
fn test_parse_crate_by_marco_expand() {
    use std::env;
    use std::path::PathBuf;
    use std::time::Duration;

    use env_logger;

    //启动日志系统
    env_logger::builder().format_timestamp_millis().init();

    let rt = MultiTaskRuntimeBuilder::default().build();

    rt.spawn(rt.alloc(), async move {
        let path = PathBuf::from(r#".\tests\pi_ui_ext"#);
        let vm_builtin_path = PathBuf::from(r#"..\..\pi_v8\vm_builtin"#);
        let crates = parse_crates(vec![PathBuf::from(r#"E:\wsl_tmp\pi_ui_render"#)],
                                  Some(vec!["style.rs".to_string()]),
                                  true).await.unwrap();
        let ts_proxy_root = PathBuf::from(r#".\tests\pi_ui_ext\ts"#);

        match create_bind_crate(path, vm_builtin_path, "0.1.0", "2018", crates.as_slice()).await {
            Err(e) => {
                panic!("Test create bind crate failed, {:?}", e);
            },
            Ok(src_path) => {
                let generater = ProxySourceGenerater::new();
                if let Err(e) = generate_crates_proxy_source(&generater, crates, src_path.clone(), ts_proxy_root.clone(), false).await {
                    panic!("Test generate proxy file failed, {:?}", e);
                }
            },
        }
    }).unwrap();

    std::thread::sleep(Duration::from_millis(1000000000));
}

//测试在导出库中使用第三方库的类型作为导出函数的参数或返回值
//方法为在导出库的导出函数使用了任意第三方库的任意类型时，使用pub关键字将这个类型单独引入，例：pub use pi_atom::Atom;
#[test]
fn test_parse_crate_depends() {
    use std::env;
    use std::path::PathBuf;
    use std::time::Duration;

    use env_logger;

    //启动日志系统
    env_logger::builder().format_timestamp_millis().init();

    let rt = MultiTaskRuntimeBuilder::default().build();

    rt.spawn(rt.alloc(), async move {
        let path = PathBuf::from(r#"E:\wsl_tmp\test_depends_ext"#);
        let vm_builtin_path = PathBuf::from(r#"..\..\pi_v8\vm_builtin"#);
        let crates = parse_crates(vec![PathBuf::from(r#".\tests\test_depends"#)],
                                  None,
                                  true).await.unwrap();
        let ts_proxy_root = PathBuf::from(r#"E:\wsl_tmp\test_depends_ext\ts"#);

        match create_bind_crate(path, vm_builtin_path, "0.1.0", "2018", crates.as_slice()).await {
            Err(e) => {
                panic!("Test create bind crate failed, {:?}", e);
            },
            Ok(src_path) => {
                let generater = ProxySourceGenerater::new();
                if let Err(e) = generate_crates_proxy_source(&generater, crates, src_path.clone(), ts_proxy_root.clone(), false).await {
                    panic!("Test generate proxy file failed, {:?}", e);
                }
            },
        }
    }).unwrap();

    std::thread::sleep(Duration::from_millis(1000000000));
}