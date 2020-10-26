use std::fs;
use std::sync::Arc;
use std::str::FromStr;
use std::io::{Error, Result, ErrorKind};
use std::path::{Path, PathBuf, Component};

use futures::future::{FutureExt, BoxFuture};
use toml;

use r#async::rt::AsyncRuntime;
use async_file::file::{create_dir, remove_file, AsyncFileOptions, WriteOptions, AsyncFile};
use bytes::{Buf, BufMut};

use crate::{WORKER_RUNTIME,
            rust_backend::{DEFAULT_DEPEND_CRATE_NAME, DEFAULT_PROXY_LIB_REGISTER_FUNCTION_NAME, DEFAULT_PROXY_FUNCTION_BLOCK_END, DEFAULT_PROXY_LIB_FILE_USED, create_proxy_rust_file, generate_rust_import, generate_rust_functions},
            ts_backend::{generate_public_exports, create_proxy_ts_file, generate_ts_import, generate_ts_impls},
            utils::{SRC_DIR_NAME, LIB_FILE_NAME, BUILD_FILE_NAME,
                    Crate, CrateInfo, ParseContext, ExportItem, Function, Generic, Type, TypeName, ProxySourceGenerater,
                    abs_path, create_tab, get_target_type_name}};

/*
* 异步创建指定名称、版本号和版本的pi_v8外部绑定库，初始化并返回库的源码路径
*/
pub(crate) async fn create_bind_crate(path: PathBuf,
                                      root: PathBuf,
                                      version: &str,
                                      edition: &str,
                                      export_crates: &[Crate]) -> Result<PathBuf> {
    let root = abs_path(root.as_path()).unwrap();
    let src_path = path.join(SRC_DIR_NAME);
    if let Err(e) = create_dir(WORKER_RUNTIME.clone(), src_path.clone()).await {
        //创建目录失败，则立即返回错误
        return Err(Error::new(ErrorKind::Other, format!("Create bind crate failed, path: {:?}, reason: {:?}", path, e)));
    }

    //移除源文件目录中的所有文件
    match fs::read_dir(src_path.clone()) {
        Err(e) => {
            //获取源文件目录中的成员失败，则立即返回错误
            return Err(Error::new(ErrorKind::Other, format!("Create bind crate failed, path: {:?}, reason: {:?}", path, e)));
        },
        Ok(mut dir) => {
            while let Some(entry) = dir.next() {
                match entry {
                    Err(e) => {
                        //获取源文件目录中的成员失败，则立即返回错误
                        return Err(Error::new(ErrorKind::Other, format!("Create bind crate failed, path: {:?}, reason: {:?}", path, e)));
                    },
                    Ok(e) => {
                        let p = e.path();
                        if p.is_file() {
                            if let Err(e) = remove_file(WORKER_RUNTIME.clone(), p.clone()).await {
                                //移除源文件目录中的文件失败，则立即返回错误
                                return Err(Error::new(ErrorKind::Other, format!("Create bind crate failed, path: {:?}, reason: {:?}", p, e)));
                            }
                        }
                    },
                }
            }
        },
    }

    //初始化构建配置
    match AsyncFile::open(WORKER_RUNTIME.clone(),
                          path.join(BUILD_FILE_NAME),
                          AsyncFileOptions::TruncateWrite).await {
        Err(e) => {
            //创建构建配置文件失败，则立即返回错误
            return Err(Error::new(ErrorKind::Other, format!("Create bind crate failed, path: {:?}, reason: {:?}", path, e)));
        },
        Ok(file) => {
            let crate_path = PathBuf::from(path.to_str().unwrap().replace(r#"\"#, "/"));
            let mut configure = CrateInfo::new(crate_path.file_name().unwrap().to_str().unwrap(),
                                           version,
                                           vec!["yineng <yineng@foxmail.com>"],
                                           edition);
            if let Err(e) = parse_crate_depends(root, export_crates, &mut configure) {
                //分析需要增加依赖的导出库失败，则立即返回
                return Err(e);
            }
            let configure_string: String = configure.into();
            let buf: Arc<[u8]> = Arc::from(configure_string.into_bytes());

            if let Err(e) = file.write(0, buf, WriteOptions::Sync(true)).await {
                //初始化构建配置文件失败，则立即返回错误
                return Err(Error::new(ErrorKind::Other, format!("Create bind crate failed, path: {:?}, reason: {:?}", path, e)));
            }
        },
    }

    Ok(src_path)
}

//分析库依赖，并将需要导入的导出库写入构建配置的依赖中
fn parse_crate_depends(root: PathBuf,
                       export_crates: &[Crate],
                       configure: &mut CrateInfo) -> Result<()> {
    //写入默认依赖
    let export_crate_name = DEFAULT_DEPEND_CRATE_NAME.to_string();
    let export_crate_path = root.join(export_crate_name.as_str());
    let mut table = toml::value::Table::new();
    table.insert("path".to_string(), toml::Value::String(export_crate_path.into_os_string().into_string().unwrap()));
    configure.append_depend("futures", toml::Value::String("0.3".to_string())); //异步库
    configure.append_depend("vm_builtin", toml::Value::Table(table)); //js虚拟机内置库

    for export_crate in export_crates {
        let package = export_crate.get_info().get_package();
        let export_crate_name = package.get_name();
        let export_crate_path = root.join(export_crate_name.as_str());

        if !export_crate_path.exists() {
            //导出库不在默认的路径下，则立即返回错误
            return Err(Error::new(ErrorKind::Other, format!("Parse crate depends failed, path: {:?}, reason: crate not exist", export_crate_path)));
        }

        let mut table = toml::value::Table::new();
        table.insert("path".to_string(), toml::Value::String(export_crate_path.into_os_string().into_string().unwrap()));
        configure.append_depend(export_crate_name.as_str(), toml::Value::Table(table));
    }

    Ok(())
}

//异步解析所有导入库中的源码，并生成pi_v8的代理源码
pub(crate) async fn generate_crates_proxy_source(generater: &ProxySourceGenerater,
                                                 import_crates: Vec<Crate>,
                                                 generate_rust_path: PathBuf,
                                                 generate_ts_path: PathBuf) -> Result<()> {
    let mut map = WORKER_RUNTIME.map();

    for import_crate in import_crates {
        let generater_copy = generater.clone();
        let generate_rust_path_copy = generate_rust_path.clone();
        let generate_ts_path_copy = generate_ts_path.clone();
        let future = async move {
            if let Err(e) = generate_crate_proxy_source(generater_copy,
                                                        &import_crate,
                                                        generate_rust_path_copy.as_path(),
                                                        generate_ts_path_copy.as_path()).await {
                //生成导入库的代理源码失败，则立即返回错误
                return Err(Error::new(ErrorKind::Other, format!("Generate proxy source failed, crate: {}, reason: {:?}", import_crate.get_info().get_package().get_name(), e)));
            }

            Ok(())
        }.boxed();

        map.join(AsyncRuntime::Multi(WORKER_RUNTIME.clone()), future);
    }

    match map.map(AsyncRuntime::Multi(WORKER_RUNTIME.clone())).await {
        Err(e) => Err(e),
        Ok(vec) => {
            //异步解析所有导入库中的源码
            let mut iter = vec.into_iter();
            while let Some(Err(e)) = iter.next() {
                return Err(e);
            }

            //完成pi_v8的所有代理文件和所有代理文件的代码的生成，则创建代理库的入口文件，并生成入口文件的代码
            let lib_path = generate_rust_path.join(LIB_FILE_NAME);
            if let Err(e) = generate_crate_proxy_lib(generater, lib_path).await {
                return Err(e);
            }

            //完成创建代理和生成代理库入口文件，则创建ts的代理库本地环境文件，并生成本地环境文件的代码
            generate_public_exports(generate_ts_path.as_path()).await
        },
    }
}

//生成代理入口文件，并写入口代码
async fn generate_crate_proxy_lib(generater: &ProxySourceGenerater,
                                  generate_lib_path: PathBuf) -> Result<()> {
    match AsyncFile::open(WORKER_RUNTIME.clone(), generate_lib_path.clone(), AsyncFileOptions::TruncateWrite).await {
        Err(e) => {
            //创建代理库入口文件失败，则立即返回错误
            Err(Error::new(ErrorKind::Other, format!("Generate proxy crate lib file failed, file: {:?}, reason: {:?}", generate_lib_path, e)))
        },
        Ok(file) => {
            //创建代理库入口文件成功
            let mut lib_content: Vec<u8> = Vec::new();

            //生成入口文件的导入和导出
            lib_content.put_slice(DEFAULT_PROXY_LIB_FILE_USED);

            let export_mods = generater.take_export_mods().await;
            for export_mod in &export_mods {
                lib_content.put_slice(("mod ".to_string() + export_mod + ";\n").as_bytes());
            }
            lib_content.put_slice("\n".as_bytes());
            for export_mod in &export_mods {
                lib_content.put_slice(("use ".to_string() + export_mod + "::*;\n").as_bytes());
            }
            lib_content.put_slice("\n".as_bytes());

            //生成入口文件的注册函数
            lib_content.put_slice(DEFAULT_PROXY_LIB_REGISTER_FUNCTION_NAME.as_bytes());

            //生成入口文件的注册静态函数的代码和注册关联函数的代码
            lib_content.put_slice((create_tab(1) + "//注册静态函数和本地对象的关联函数\n").as_bytes());
            let static_methods = generater.take_static_methods().await;
            for static_method in static_methods {
                lib_content.put_slice((create_tab(1) + "register_native_object_static_method(" + static_method.as_str() + ");\n").as_bytes());
            }
            lib_content.put_slice("\n".as_bytes());

            //生成入口文件的注册异步静态函数的代码和注册异步关联函数的代码
            lib_content.put_slice((create_tab(1) + "//注册异步静态函数和本地对象的异步关联函数\n").as_bytes());
            let async_static_methods = generater.take_async_static_methods().await;
            for async_static_method in async_static_methods {
                lib_content.put_slice((create_tab(1) + "register_native_object_async_static_method(" + async_static_method.as_str() + ");\n").as_bytes());
            }
            lib_content.put_slice("\n".as_bytes());

            //生成入口文件的注册本地对象方法的代码
            lib_content.put_slice((create_tab(1) + "//注册本地对象的方法\n").as_bytes());
            let methods = generater.take_methods().await;
            for method in methods {
                lib_content.put_slice((create_tab(1) + "register_native_object_method(" + method.as_str() + ");\n").as_bytes());
            }
            lib_content.put_slice("\n".as_bytes());

            //生成入口文件的注册本地对象异步方法的代码
            lib_content.put_slice((create_tab(1) + "//注册本地对象的异步方法\n").as_bytes());
            let async_methods = generater.take_async_methods().await;
            for async_method in async_methods {
                lib_content.put_slice((create_tab(1) + "register_native_object_async_method(" + async_method.as_str() + ");\n").as_bytes());
            }

            lib_content.put_slice(DEFAULT_PROXY_FUNCTION_BLOCK_END);

            //将入口文件内容写入文件
            let buf = Arc::from(lib_content);
            if let Err(e) = file.write(0, buf, WriteOptions::SyncAll(true)).await {
                return Err(Error::new(ErrorKind::Other, format!("Write proxy lib file failed, file: {:?}, reason: {:?}", generate_lib_path, e)));
            }

            Ok(())
        },
    }
}

//生成导入库中有导出声明的文件的代理文件，并写入代码
async fn generate_crate_proxy_source(generater: ProxySourceGenerater,
                                     import_crate: &Crate,
                                     generate_rust_path: &Path,
                                     generate_ts_path: &Path) -> Result<()> {
    let crate_name = import_crate.get_info().get_package().get_name();

    //创建指定导入库的所有代理文件，并生成定导入库的所有代理文件的代码
    for source in import_crate.get_source() {
        if source.get_exports().len() == 0 {
            //没有导出条目，则忽略
            continue;
        }

        //有导出条目
        match create_proxy_rust_file(&generater, crate_name.clone(), source, generate_rust_path).await {
            Some(Err(e)) => {
                //创建代理Rust文件失败，则立即返回错误
                return Err(Error::new(ErrorKind::Other, format!("Create proxy rust file failed, crate: {}, source path: {:?}, reason: {:?}", crate_name, source.get_origin(), e)));
            },
            Some(Ok(rust_file)) => {
                match create_proxy_ts_file(crate_name.clone(), source, generate_ts_path).await {
                    Some(Err(e)) => {
                        return Err(e);
                    },
                    Some(Ok((ts_file_path, ts_file))) => {
                        if let Err(e) = write_proxy_file(&generater,
                                                         crate_name.clone(),
                                                         source,
                                                         rust_file,
                                                         ts_file_path,
                                                         ts_file).await {
                            //写代理文件失败，则立即返回错误
                            return Err(Error::new(ErrorKind::Other, format!("Create proxy file failed, crate: {}, source path: {:?}, reason: {:?}", crate_name, source.get_origin(), e)));
                        }
                    },
                    None => {
                        //导出条目中没有导出任何方法或静态函数，则不需要创建代理文件，并继续下一个导出条目的处理
                        continue;
                    },
                }
            },
            None => {
                //导出条目中没有导出任何方法或静态函数，则不需要创建代理文件，并继续下一个导出条目的处理
                continue;
            },
        }
    }

    Ok(())
}

//写入导入库的导出文件中的导出条目到代理文件中
pub async fn write_proxy_file(generater: &ProxySourceGenerater,
                              crate_name: String,
                              source: &ParseContext,
                              rust_file: AsyncFile<()>,
                              ts_file_path: PathBuf,
                              ts_file: AsyncFile<()>) -> Result<()> {
    //写代理导出文件的Rust文件
    let source_path = source.get_origin();
    let mut components = source_path.components();

    let mut b = false;
    let mut path_buf = PathBuf::from(crate_name);
    while let Some(c) = components.next() {
        match c {
            Component::Normal(str) => {
                if let Some("src") = str.to_str() {
                    if !b {
                        //如果是首次出现的src目录，则设置状态，并继续下一个路径的分析
                        b = true;
                        continue;
                    }
                } else if !b {
                    continue;
                }

                //记录路径
                path_buf = path_buf.join(str);
            },
            _ => continue,
        }
    }

    let filename = if cfg!(windows) {
        path_buf.to_str().unwrap().replace(r#"\"#, "_")
    } else {
        path_buf.to_str().unwrap().replace(r#"/"#, "_")
    };

    let mut source_content = generate_rust_import(path_buf.clone(), source);
    if let Err(e) = generate_rust_functions(generater, source, &mut source_content).await {
        return Err(Error::new(ErrorKind::Other, format!("Write proxy rust file failed, file: {}, reason: {:?}", filename, e)));
    }

    let buf = Arc::from(source_content);
    if let Err(e) = rust_file.write(0, buf, WriteOptions::SyncAll(true)).await {
        return Err(Error::new(ErrorKind::Other, format!("Write proxy rust file failed, file: {}, reason: {:?}", filename, e)));
    }

    //写代理导出文件的ts文件
    let mut source_content = generate_ts_import(path_buf.clone());
    if let Err(e) = generate_ts_impls(generater, source, &mut source_content).await {
        return Err(Error::new(ErrorKind::Other, format!("Write proxy ts file failed, file: {:?}, reason: {:?}", ts_file_path, e)));
    }

    let buf = Arc::from(source_content);
    if let Err(e) = ts_file.write(0, buf, WriteOptions::SyncAll(true)).await {
        return Err(Error::new(ErrorKind::Other, format!("Write proxy ts file failed, file: {:?}, reason: {:?}", ts_file_path, e)));
    }

    Ok(())
}