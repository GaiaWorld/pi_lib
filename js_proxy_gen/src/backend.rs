use std::fs;
use std::sync::Arc;
use std::str::FromStr;
use std::io::{Error, Result, ErrorKind};
use std::path::{Path, PathBuf, Component};
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::future::{FutureExt, BoxFuture};
use toml;

use r#async::{lock::mutex_lock::Mutex,
              rt::AsyncRuntime};
use async_file::file::{create_dir, remove_file, AsyncFileOptions, WriteOptions, AsyncFile};
use bytes::{Buf, BufMut};

use crate::{WORKER_RUNTIME,
            utils::{SRC_DIR_NAME, LIB_FILE_NAME, BUILD_FILE_NAME,
                    Crate, CrateInfo, ParseContext, ExportItem, Function, Generic, Type, TypeName,
                    abs_path, create_tab, get_target_type_name}};

/*
* 导出的外部绑定库的默认相对路径的根
*/
#[cfg(target_os = "windows")]
pub const DEFAULT_EXPORT_CRATES_PATH_ROOT: &str = r#"..\..\"#;
#[cfg(target_os = "unix")]
pub const DEFAULT_EXPORT_CRATES_PATH_ROOT: &str = "../../";

/*
* 默认的依赖库名
*/
#[cfg(target_os = "windows")]
const DEFAULT_DEPEND_CRATE_NAME: &str = r#"pi_v8\vm_builtin"#;
#[cfg(target_os = "unix")]
const DEFAULT_DEPEND_CRATE_NAME: &str = "pi_v8/vm_builtin";

/*
* 默认代理入口文件导入的类型
*/
const DEFAULT_PROXY_LIB_FILE_USED: &[u8] = b"use vm_builtin::external::{register_native_object_static_method,\n\t\t\t\t\t\t\tregister_native_object_async_static_method,\n\t\t\t\t\t\t\tregister_native_object_method,\n\t\t\t\t\t\t\tregister_native_object_async_method};\n\n";

/*
* 默认的代理入口文件注册代理函数的函数签名
*/
const DEFAULT_PROXY_LIB_REGISTER_FUNCTION_NAME: &str = "/**\n * 注册所有自动导入的外部扩展库中声明的导出函数\n */\npub fn register_ext_functions() {\n";

/*
* 默认代理Rust文件导入的类型
*/
const DEFAULT_PROXY_RUST_FILE_USED: &[u8] = b"use std::any::Any;\nuse std::sync::Arc;\n\nuse futures::future::{FutureExt, BoxFuture};\n\nuse vm_builtin::{buffer::NativeArrayBuffer, external::{NativeObjectAsyncTaskSpawner, NativeObjectAsyncReply, NativeObjectValue, NativeObjectArgs, NativeObject}};\n\n";

/*
* 默认的代理函数签名前缀
*/
const DEFAULT_PROXY_FUNCTION_SING_PREFIX: &[u8] = b"pub fn ";

/*
* 默认的静态代理函数名前缀
*/
const DEFAULT_STATIC_PROXY_FUNCTION_NAME_PREFIX: &[u8] = b"static_call_";

/*
* 默认的异步静态代理函数名前缀
*/
const DEFAULT_ASYNC_STATIC_PROXY_FUNCTION_NAME_PREFIX: &[u8] = b"async_static_call_";

/*
* 默认的代理函数名前缀
*/
const DEFAULT_PROXY_FUNCTION_NAME_PREFIX: &[u8] = b"call_";

/*
* 默认的静态代理函数名前缀
*/
const DEFAULT_ASYNC_PROXY_FUNCTION_NAME_PREFIX: &[u8] = b"async_call_";

/*
* 默认的静态代理函数签名后缀
*/
const DEFAULT_STATIC_PROXY_FUNCTION_SIGN_SUFFIX: &[u8] = b"(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {\n";

/*
* 默认的异步静态代理函数签名后缀
*/
const DEFAULT_ASYNC_STATIC_PROXY_FUNCTION_SIGN_SUFFIX: &[u8] = b"(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {\n";

/*
* 默认的代理函数签名后缀
*/
const DEFAULT_PROXY_FUNCTION_SIGN_SUFFIX: &[u8] = b"(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {\n";

/*
* 默认的静态代理函数签名后缀
*/
const DEFAULT_ASYNC_PROXY_FUNCTION_SIGN_SUFFIX: &[u8] = b"(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {\n";

/*
* 默认的代理函数块结束符
*/
const DEFAULT_PROXY_FUNCTION_BLOCK_END: &[u8] = b"}\n\n";

/*
* 默认的实参名称前缀
*/
const DEFAULT_ARGUMENT_NAME_PREFIX: &str = "arg_";

/*
* 异步创建指定名称、版本号和版本的pi_v8外部绑定库，初始化并返回库的源码路径
*/
pub async fn create_bind_crate(path: PathBuf,
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

/*
* 代理源码生成器
*/
#[derive(Clone)]
pub struct ProxySourceGenerater {
    static_method_index:        Arc<AtomicUsize>,           //同步静态代理方法序号
    async_static_method_index:  Arc<AtomicUsize>,           //异步静态代理方法序号
    method_index:               Arc<AtomicUsize>,           //同步代理方法序号
    async_method_index:         Arc<AtomicUsize>,           //异步代理方法序号
    export_mods:                Arc<Mutex<Vec<String>>>,    //需要在lib中导出的模块名列表
    static_methods:             Arc<Mutex<Vec<String>>>,    //需要注册的同步静态代理方法名列表
    async_static_methods:       Arc<Mutex<Vec<String>>>,    //需要注册的异步静态代理方法名列表
    methods:                    Arc<Mutex<Vec<String>>>,    //需要注册的同步代理方法名列表
    async_methods:              Arc<Mutex<Vec<String>>>,    //需要注册的异步代理方法名列表
}

unsafe impl Send for ProxySourceGenerater {}
unsafe impl Sync for ProxySourceGenerater {}

/*
* 代理源码生成器同步方法
*/
impl ProxySourceGenerater {
    //构建代理源码生成器
    pub fn new() -> Self {
        ProxySourceGenerater {
            static_method_index: Arc::new(AtomicUsize::new(0)),
            async_static_method_index: Arc::new(AtomicUsize::new(0)),
            method_index: Arc::new(AtomicUsize::new(0)),
            async_method_index: Arc::new(AtomicUsize::new(0)),
            export_mods: Arc::new(Mutex::new(Vec::new())),
            static_methods: Arc::new(Mutex::new(Vec::new())),
            async_static_methods: Arc::new(Mutex::new(Vec::new())),
            methods: Arc::new(Mutex::new(Vec::new())),
            async_methods: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/*
* 代理源码生成器异步方法
*/
impl ProxySourceGenerater {
    //获取需要在lib中导出的模块名列表
    pub async fn take_export_mods(&self) -> Vec<String> {
        self.export_mods.lock().await.clone()
    }

    //追加需要在lib中导出的模块名
    pub async fn append_export_mod(&self, name: String) {
        self.export_mods.lock().await.push(name);
    }

    //获取需要注册的同步静态代理方法名列表
    pub async fn take_static_methods(&self) -> Vec<String> {
        self.static_methods.lock().await.clone()
    }

    //追加需要注册的同步静态代理方法名，返回分配的同步静态代理方法序号
    pub async fn append_static_method(&self, name: String) -> usize {
        let mut static_methods = self.static_methods.lock().await;
        let method_index = self.static_method_index.fetch_add(1, Ordering::Relaxed);
        let method_name = name + method_index.to_string().as_str();
        static_methods.push(method_name);
        method_index
    }

    //获取需要注册的异步静态代理方法名列表
    pub async fn take_async_static_methods(&self) -> Vec<String> {
        self.async_static_methods.lock().await.clone()
    }

    //追加需要注册的异步静态代理方法名，返回分配的异步静态代理方法序号
    pub async fn append_async_static_method(&self, name: String) -> usize {
        let mut async_static_methods = self.async_static_methods.lock().await;
        let method_index = self.async_static_method_index.fetch_add(1, Ordering::Relaxed);
        let method_name = name + method_index.to_string().as_str();
        async_static_methods.push(method_name);
        method_index
    }

    //获取需要注册的同步代理方法名列表
    pub async fn take_methods(&self) -> Vec<String> {
        self.methods.lock().await.clone()
    }

    //追加需要注册的同步代理方法名，返回分配的同步代理方法序号
    pub async fn append_method(&self, name: String) -> usize {
        let mut methods = self.methods.lock().await;
        let method_index = self.method_index.fetch_add(1, Ordering::Relaxed);
        let method_name = name + method_index.to_string().as_str();
        methods.push(method_name);
        method_index
    }

    //获取需要注册的异步代理方法名列表
    pub async fn take_async_methods(&self) -> Vec<String> {
        self.async_methods.lock().await.clone()
    }

    //追加需要注册的异步代理方法名，返回分配的异步代理方法序号
    pub async fn append_async_method(&self, name: String) -> usize {
        let mut async_methods = self.async_methods.lock().await;
        let method_index = self.async_method_index.fetch_add(1, Ordering::Relaxed);
        let method_name = name + method_index.to_string().as_str();
        async_methods.push(method_name);
        method_index
    }
}

//异步解析所有导入库中的源码，并生成pi_v8的代理源码
pub async fn generate_crates_proxy_source(generater: &ProxySourceGenerater,
                                          import_crates: Vec<Crate>,
                                          generate_rust_path: PathBuf,
                                          generate_js_path: PathBuf) -> Result<()> {
    let mut map = WORKER_RUNTIME.map();

    for import_crate in import_crates {
        let generater_copy = generater.clone();
        let generate_rust_path_copy = generate_rust_path.clone();
        let generate_js_path_copy = generate_js_path.clone();
        let future = async move {
            if let Err(e) = generate_crate_proxy_source(generater_copy,
                                                        &import_crate,
                                                        generate_rust_path_copy.as_path(),
                                                        generate_js_path_copy.as_path()).await {
                //生成导入库的代理源码失败，则立即返回错误
                return Err(Error::new(ErrorKind::Other, format!("Generate proxy source failed, crate: {}, reason: {:?}", import_crate.get_info().get_package().get_name(), e)));
            }

            Ok(())
        }.boxed();

        map.join(AsyncRuntime::Multi(WORKER_RUNTIME.clone()), future);
    }

    match map.map(AsyncRuntime::Multi(WORKER_RUNTIME.clone()), true).await {
        Err(e) => Err(e),
        Ok(vec) => {
            //异步解析所有导入库中的源码
            let mut iter = vec.into_iter();
            while let Some(Err(e)) = iter.next() {
                return Err(e);
            }

            //完成pi_v8的所有代理文件和所有代理文件的代码的生成，则创建代理库的入口文件，并生成入口文件的代码
            let lib_path = generate_rust_path.join(LIB_FILE_NAME);
            generate_crate_proxy_lib(generater, lib_path).await
        },
    }
}

//生成代理入口文件，并写入代码
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
                                     generate_js_path: &Path) -> Result<()> {
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
            Some(Ok(file)) => {
                if let Err(e) = write_proxy_rust_file(&generater, crate_name.clone(), source, file).await {
                    //写代理Rust文件失败，则立即返回错误
                    return Err(Error::new(ErrorKind::Other, format!("Create proxy rust file failed, crate: {}, source path: {:?}, reason: {:?}", crate_name, source.get_origin(), e)));
                }
            },
            None => {
                //导出条目中没有导出任何方法或静态函数，则不需要创建Rust代理文件，并继续下一个导出条目的处理
                continue;
            },
        }
    }

    Ok(())
}

//在指定路径下创建代理的Rust文件，并返回异步文件句柄
async fn create_proxy_rust_file(generater: &ProxySourceGenerater,
                                crate_name: String,
                                source: &ParseContext,
                                generate_rust_path: &Path) -> Option<Result<AsyncFile<()>>> {
    //生成文件名
    let mut b = false;
    for export_item in source.get_exports() {
        match export_item {
            ExportItem::StructItem(s) => {
                if s.get_trait_impls().is_some() || s.get_impls().is_some() {
                    //有导出任意方法，则继续创建代理的Rust文件
                    b = true;
                    break;
                }
            },
            ExportItem::EnumItem(e) => {
                if e.get_trait_impls().is_some() || e.get_impls().is_some() {
                    //有导出任意方法，则继续创建代理的Rust文件
                    b = true;
                    break;
                }
            },
            ExportItem::FunctionItem(_) => {
                //有导出任意静态函数，则继续创建代理的Rust文件
                b = true;
                break;
            },
            _ => (),
        }
    }
    if !b {
        //未导出任何的方法或静态函数，则忽略，并立即退出
        return None;
    }

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

    //记录需要在lib中导出的代理Rust文件模块名
    let mut mod_path = PathBuf::from(filename.as_str());
    mod_path.set_extension("");
    generater.append_export_mod(mod_path.to_str().unwrap().to_string()).await;

    //创建文件
    let file_path = generate_rust_path.join(filename);
    Some(AsyncFile::open(WORKER_RUNTIME.clone(), file_path, AsyncFileOptions::TruncateWrite).await)
}

//写入导入库的导出文件中的导出条目到代理的Rust文件中
pub async fn write_proxy_rust_file(generater: &ProxySourceGenerater,
                                   crate_name: String,
                                   source: &ParseContext,
                                   file: AsyncFile<()>) -> Result<()> {
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
        return Err(Error::new(ErrorKind::Other, format!("Write proxy file failed, file: {}, reason: {:?}", filename, e)));
    }

    let buf = Arc::from(source_content);
    if let Err(e) = file.write(0, buf, WriteOptions::SyncAll(true)).await {
        return Err(Error::new(ErrorKind::Other, format!("Write proxy file failed, file: {}, reason: {:?}", filename, e)));
    }

    Ok(())
}

//生成Rust文件的导入
fn generate_rust_import(mut path_buf: PathBuf,
                        source: &ParseContext) -> Vec<u8> {
    let mut source_content = Vec::from(DEFAULT_PROXY_RUST_FILE_USED);

    path_buf.set_extension(""); //移除文件扩展名
    let source_string = if cfg!(windows) {
        "use ".to_string() + path_buf.to_str().unwrap().replace(r#"\"#, "::").as_str() + "::*;\n\n"
    } else {
        "use ".to_string() + path_buf.to_str().unwrap().replace(r#"/"#, "::").as_str() + "::*;\n\n"
    };
    source_content.put_slice(source_string.as_bytes());

    source_content
}

//生成Rust文件的所有代理函数
async fn generate_rust_functions(generater: &ProxySourceGenerater,
                                 source: &ParseContext,
                                 source_content: &mut Vec<u8>) -> Result<()> {
    for export_item in source.get_exports() {
        match export_item {
            ExportItem::StructItem(struct_item) => {
                let items = struct_item.get_specific_structs();
                let struct_items = if let Some(specific_struct_item) = &items {
                    specific_struct_item.iter().collect()
                } else {
                    vec![struct_item]
                };

                //生成具体类型的结构体的实现代码
                for struct_item in struct_items {
                    let struct_target = struct_item.get_name();
                    let struct_generic = struct_item.get_generic();

                    //生成导入库的源码文件的结构体实现的所有导出的trait方法
                    for trait_impl in struct_item.get_trait_impls() {
                        for (trait_name, functions) in trait_impl.get_ref() {
                            for function in functions {
                                if let Err(e) = generate_rust_function(generater, struct_target, struct_generic, function, source_content).await {
                                    return Err(Error::new(ErrorKind::Other, format!("Generate rust proxy function failed, struct: {}, trait: {}, method: {}, reason: {:?}", struct_item.get_name().unwrap(), trait_name, function.get_name().unwrap(), e)));
                                }
                            }
                        }
                    }

                    //生成导入库的源码文件的结构体实现的所有导出的方法
                    for struct_impl in struct_item.get_impls() {
                        for function in struct_impl.get_ref() {
                            if let Err(e) = generate_rust_function(generater, struct_target, struct_generic, function, source_content).await {
                                return Err(Error::new(ErrorKind::Other, format!("Generate rust proxy function failed, struct: {}, method: {}, reason: {:?}", struct_item.get_name().unwrap(), function.get_name().unwrap(), e)));
                            }
                        }
                    }
                }
            },
            ExportItem::EnumItem(enum_item) => {
                let items = enum_item.get_specific_enums();
                let enum_items = if let Some(specific_enum_item) = &items {
                    specific_enum_item.iter().collect()
                } else {
                    vec![enum_item]
                };

                //生成具体类型的枚举的实现代码
                for enum_item in enum_items {
                    let enum_target = enum_item.get_name();
                    let enum_generic = enum_item.get_generic();

                    //生成导入库的源码文件的枚举实现的所有导出的trait方法
                    for trait_impl in enum_item.get_trait_impls() {
                        for (trait_name, functions) in trait_impl.get_ref() {
                            for function in functions {
                                if let Err(e) = generate_rust_function(generater, enum_target, enum_generic, function, source_content).await {
                                    return Err(Error::new(ErrorKind::Other, format!("Generate rust proxy function failed, struct: {}, trait: {}, method: {}, reason: {:?}", enum_item.get_name().unwrap(), trait_name, function.get_name().unwrap(), e)));
                                }
                            }
                        }
                    }

                    //生成导入库的源码文件的枚举实现的所有导出的方法
                    for enum_impl in enum_item.get_impls() {
                        for function in enum_impl.get_ref() {
                            if let Err(e) = generate_rust_function(generater, enum_target, enum_generic, function, source_content).await {
                                return Err(Error::new(ErrorKind::Other, format!("Generate rust proxy function failed, enum: {}, method: {}, reason: {:?}", enum_item.get_name().unwrap(), function.get_name().unwrap(), e)));
                            }
                        }
                    }
                }
            },
            ExportItem::FunctionItem(function) => {
                let items = function.get_specific_functions();
                let functions = if let Some(specific_function) = &items {
                    specific_function.iter().collect()
                } else {
                    vec![function]
                };

                //生成具体类型的静态函数的代码
                for function in functions {
                    if let Err(e) = generate_rust_function(generater, None, None, function, source_content).await {
                        return Err(Error::new(ErrorKind::Other, format!("Generate rust proxy function failed, static function: {}, reason: {:?}", function.get_name().unwrap(), e)));
                    }
                }
            },
            _ => (),
        }
    }

    Ok(())
}

//生成Rust文件的代理函数
async fn generate_rust_function(generater: &ProxySourceGenerater,
                                target: Option<&String>,
                                generic: Option<&Generic>,
                                function: &Function,
                                source_content: &mut Vec<u8>) -> Result<()> {
    generate_rust_function_comment(function, source_content);

    if function.is_static() {
        if function.is_async() {
            //生成异步静态函数，则录异步静态代理函数名，并分配异步静态代理函数序号
            let index =
                generater
                    .append_async_static_method(String::from_utf8(DEFAULT_ASYNC_STATIC_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap()).await;
            generate_async_static_function(target, generic, function, index, source_content)
        } else {
            //生成静态函数，则记录静态代理函数名，并分配静态代理函数序号
            let index =
                generater
                    .append_static_method(String::from_utf8(DEFAULT_STATIC_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap()).await;
            generate_static_function(target, generic, function, index, source_content)
        }
    } else {
        if function.is_async() {
            //生成异步函数，则记录异步代理函数名，并分配异步代理函数序号
            let index =
                generater
                    .append_async_method(String::from_utf8(DEFAULT_ASYNC_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap()).await;
            generate_async_function(target, generic, function, index, source_content)
        } else {
            //生成函数，则记录代理函数名，并分配代理函数序号
            let index =
                generater
                    .append_method(String::from_utf8(DEFAULT_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap()).await;
            generate_function(target, generic, function, index, source_content)
        }
    }
}

//生成Rust文件的代理函数的注释
fn generate_rust_function_comment(function: &Function,
                                  source_content: &mut Vec<u8>) {
    if let Some(doc) = function.get_doc() {
        //有导出文档，则写入注释
        source_content.put_slice(b"/**\n");   //写入起始注释
        for comment in doc.get_ref() {
            //写入注释内容
            let comment_str = (" *".to_string() + comment.replace(r#"""#, "").replace(r#"\r"#, "").as_str() + "\n");
            source_content.put_slice(comment_str.as_bytes());
        }
        source_content.put_slice(b" */\n");   //写入结尾注释
    }
}

//生成静态代理函数
fn generate_static_function(target: Option<&String>,
                            generic: Option<&Generic>,
                            function: &Function,
                            index: usize,
                            source_content: &mut Vec<u8>) -> Result<()> {
    //生成静态代理函数签名
    source_content.put_slice(DEFAULT_PROXY_FUNCTION_SING_PREFIX);
    let static_function_name = String::from_utf8(DEFAULT_STATIC_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap() + index.to_string().as_str();
    source_content.put_slice(static_function_name.as_bytes());
    source_content.put_slice(DEFAULT_STATIC_PROXY_FUNCTION_SIGN_SUFFIX);

    //生成静态代理函数的实现
    let level = 1; //默认的代码格式层数
    if let Err(e) = generate_function_call(target, generic, function, level, source_content) {
        return Err(e);
    }

    //结束静态代理函数的生成
    source_content.put_slice(DEFAULT_PROXY_FUNCTION_BLOCK_END);

    Ok(())
}

//生成异步静态代理函数
fn generate_async_static_function(target: Option<&String>,
                                  generic: Option<&Generic>,
                                  function: &Function,
                                  index: usize,
                                  source_content: &mut Vec<u8>) -> Result<()> {
    //生成异步静态代理函数签名
    source_content.put_slice(DEFAULT_PROXY_FUNCTION_SING_PREFIX);
    let async_static_function_name = String::from_utf8(DEFAULT_ASYNC_STATIC_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap() + index.to_string().as_str();
    source_content.put_slice(async_static_function_name.as_bytes());
    source_content.put_slice(DEFAULT_ASYNC_STATIC_PROXY_FUNCTION_SIGN_SUFFIX);

    //生成异步静态代理函数的实现
    let level = 1; //默认的代码格式层数
    source_content.put_slice((create_tab(level) + "let task = async move {\n").as_bytes());
    if let Err(e) = generate_function_call(target, generic, function, level + 1, source_content) {
        return Err(e);
    }
    source_content.put_slice((create_tab(level) + "}.boxed();\n").as_bytes());
    source_content.put_slice((create_tab(level) + "spawner(task);\n").as_bytes());

    //结束异步静态代理函数的生成
    source_content.put_slice(DEFAULT_PROXY_FUNCTION_BLOCK_END);

    Ok(())
}

//生成动态代理函数
fn generate_function(target: Option<&String>,
                     generic: Option<&Generic>,
                     function: &Function,
                     index: usize,
                     source_content: &mut Vec<u8>) -> Result<()> {
    //生成代理函数签名
    source_content.put_slice(DEFAULT_PROXY_FUNCTION_SING_PREFIX);
    let function_name = String::from_utf8(DEFAULT_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap() + index.to_string().as_str();
    source_content.put_slice(function_name.as_bytes());
    source_content.put_slice(DEFAULT_PROXY_FUNCTION_SIGN_SUFFIX);

    //生成代理函数的实现
    let level = 1; //默认的代码格式层数
    if let Err(e) = generate_function_call(target, generic, function, level, source_content) {
        return Err(e);
    }


    //结束代理函数的生成
    source_content.put_slice(DEFAULT_PROXY_FUNCTION_BLOCK_END);

    Ok(())
}

//生成异步动态代理函数
fn generate_async_function(target: Option<&String>,
                           generic: Option<&Generic>,
                           function: &Function,
                           index: usize,
                           source_content: &mut Vec<u8>) -> Result<()> {
    //生成异步代理函数签名
    source_content.put_slice(DEFAULT_PROXY_FUNCTION_SING_PREFIX);
    let async_function_name = String::from_utf8(DEFAULT_ASYNC_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap() + index.to_string().as_str();
    source_content.put_slice(async_function_name.as_bytes());
    source_content.put_slice(DEFAULT_ASYNC_PROXY_FUNCTION_SIGN_SUFFIX);

    //生成异步代理函数的实现
    let level = 1; //默认的代码格式层数
    source_content.put_slice((create_tab(level) + "let task = async move {\n").as_bytes());
    if let Err(e) = generate_function_call(target, generic, function, level + 1, source_content) {
        return Err(e);
    }
    source_content.put_slice((create_tab(level) + "}.boxed();\n").as_bytes());
    source_content.put_slice((create_tab(level) + "spawner(task);\n").as_bytes());


    //结束异步代理函数的生成
    source_content.put_slice(DEFAULT_PROXY_FUNCTION_BLOCK_END);

    Ok(())
}

//生成调用指定参数的函数调用代码
fn generate_function_call(target: Option<&String>,
                          generic: Option<&Generic>,
                          function: &Function,
                          level: isize,
                          source_content: &mut Vec<u8>) -> Result<()> {
    //写入获取代理函数入参的代码
    if (function.is_static() && function.get_input().is_some())
        || (!function.is_static() && function.get_input().unwrap().len() > 1) {
        //函数有入参，不包括接收器参数
        let code = create_tab(level) + "let args = args.get_args().unwrap();\n\n";
        source_content.put_slice(code.as_bytes());
    }

    let func_name = function.get_name().unwrap();
    if let Some(func_args) = function.get_input() {
        //有参数
        let args = if let Some(target_name) = target {
            //指定目标的函数
            let args = func_args.get_ref();
            if function.is_static() {
                //关联函数
                func_args.get_ref().to_vec()
            } else {
                //方法
                match args[0].0.as_str() {
                    "&self" => {
                        //参数是方法接收器的只读引用
                        source_content.put_slice((create_tab(level) + "let obj_arc = obj.get_ref::<" + target_name.as_str() + ">().unwrap().upgrade().unwrap();\n").as_bytes());
                        source_content.put_slice((create_tab(level) + "let self_obj = obj_arc.as_ref();\n").as_bytes());
                    },
                    "&mut self" => {
                        //参数是方法接收器的可写引用
                        source_content.put_slice((create_tab(level) + "let self_obj = obj.get_mut::<" + target_name.as_str() + ">().unwrap();\n").as_bytes());
                    },
                    arg_name@"self" | arg_name@"mut self" => {
                        //参数是方法接收器的所有权，则立即返回错误
                        return Err(Error::new(ErrorKind::Other, format!("Generate fucntion call failed, function: {}, arg: {}, reason: not allowed take owner of receiver type", func_name, arg_name)));
                    },
                    _ => {
                        //不应该执行此分支
                        unimplemented!();
                    },
                };

                (&args[1..]).to_vec()
            }
        } else {
            //静态函数
            func_args.get_ref().to_vec()
        };

        let mut arg_names = Vec::with_capacity(args.len());
        if let Err(e) = generate_function_call_args(target, generic, function, &args[..], 0, level, &mut arg_names, source_content) {
            return Err(e);
        }
    } else {
        //无参数
        let mut arg_names = Vec::new();
        if let Err(e) = generate_function_call_args(target, generic, function, &vec![], 0, level, &mut arg_names, source_content) {
            return Err(e);
        }
    }

    Ok(())
}

//生成函数调用代码的实参列表
fn generate_function_call_args(target: Option<&String>,
                               generic: Option<&Generic>,
                               function: &Function,
                               args: &[(String, Type)],
                               index: usize,
                               level: isize,
                               arg_names: &mut Vec<String>,
                               source_content: &mut Vec<u8>) -> Result<()> {
    let func_name = function.get_name().unwrap().clone();
    let origin_arg_name = if let Some((origin_arg_name, _)) = args.get(index) {
        origin_arg_name
    } else {
        //实参列表已生成完成，则生成函数调用代码
        return generate_call_function(target, generic, function, level, arg_names, source_content, &func_name);
    };
    let arg_name =  DEFAULT_ARGUMENT_NAME_PREFIX.to_string() + index.to_string().as_str();
    let arg_type = &args[index].1; //获取指定形参的类型
    let arg_type_name = &arg_type.get_type_name(); //获取指定形参的类型名称

    //首先在函数声明的泛型参数中检查参数类型
    let func_generic = function.get_generic();
    if let Some(func_generic) = func_generic {
        for (generic_name, generic_types) in func_generic.get_ref() {
            if arg_type_name.get_name() == generic_name {
                //参数是函数声明的泛型，则生成匹配开始代码
                source_content.put_slice((create_tab(level) + "match &args[" + index.to_string().as_str() + "] {\n").as_bytes());

                //获取函数声明的泛型的具体类型，并生成匹配具体类型的代码
                for generic_type in generic_types {
                    arg_names.push(arg_name.clone()); //记录指定实参的名称
                    if let Err(e) = generate_function_call_args_match_cause(target, generic, function, args, index, level + 1, arg_names, source_content, &func_name, origin_arg_name, &arg_name, &arg_type, arg_type_name, generic_type) {
                        return Err(e);
                    }
                    let _ = arg_names.pop(); //移除多余实参的名称
                }

                //生成剩余匹配项和匹配结束的代码
                source_content.put_slice((create_tab(level + 1) + "_ => {\n").as_bytes());
                if function.is_async() {
                    //生成异步返回错误的代码
                    source_content.put_slice((create_tab(level + 2) + format!("reply(Err(NativeObjectValue::Str(\"Invalid type of {}th parameter\".to_string())));\n", index).as_str()).as_bytes());
                } else {
                    //生成同步返回错误的代码
                    source_content.put_slice((create_tab(level + 2) + format!("return Some(Err(\"Invalid type of {}th parameter\".to_string()));\n", index).as_str()).as_bytes());
                }
                source_content.put_slice((create_tab(level + 1) + "},\n").as_bytes());
                source_content.put_slice((create_tab(level) + "}\n").as_bytes());

                return Ok(());
            }
        }
    }

    //然后在对象声明的泛型参数中检查参数类型
    if let Some(generic_) = generic {
        for (generic_name, generic_types) in generic_.get_ref() {
            if arg_type_name.get_name() == generic_name {
                //参数是函数声明的泛型，则生成匹配开始代码
                source_content.put_slice((create_tab(level) + "match &args[" + index.to_string().as_str() + "] {\n").as_bytes());

                //获取函数声明的泛型的具体类型，并生成匹配具体类型的代码
                for generic_type in generic_types {
                    arg_names.push(arg_name.clone()); //记录指定实参的名称
                    if let Err(e) = generate_function_call_args_match_cause(target, generic, function, args, index, level + 1, arg_names, source_content, &func_name, origin_arg_name, &arg_name, &arg_type, arg_type_name, generic_type) {
                        return Err(e);
                    }
                    let _ = arg_names.pop(); //移除多余实参的名称
                }

                //生成剩余匹配项和匹配结束的代码
                source_content.put_slice((create_tab(level + 1) + "_ => {\n").as_bytes());
                if function.is_async() {
                    //生成异步返回错误的代码
                    source_content.put_slice((create_tab(level + 2) + format!("reply(Err(NativeObjectValue::Str(\"Invalid type of {}th parameter\".to_string())));\n", index).as_str()).as_bytes());
                } else {
                    //生成同步返回错误的代码
                    source_content.put_slice((create_tab(level + 2) + format!("return Some(Err(\"Invalid type of {}th parameter\".to_string()));\n", index).as_str()).as_bytes());
                }
                source_content.put_slice((create_tab(level + 1) + "},\n").as_bytes());
                source_content.put_slice((create_tab(level) + "}\n").as_bytes());

                return Ok(());
            }
        }
    }

    //参数类型是具体类型，则生成匹配开始代码
    source_content.put_slice((create_tab(level) + "match &args[" + index.to_string().as_str() + "] {\n").as_bytes());

    arg_names.push(arg_name.clone()); //记录指定实参的名称
    if let Err(e) = generate_function_call_args_match_cause(target, generic, function, args, index, level + 1, arg_names, source_content, &func_name, origin_arg_name, &arg_name, &arg_type, arg_type_name, arg_type_name) {
        return Err(e);
    }
    let _ = arg_names.pop(); //移除多余实参的名称

    //生成剩余匹配项和匹配结束的代码
    source_content.put_slice((create_tab(level + 1) + "_ => {\n").as_bytes());
    if function.is_async() {
        //生成异步返回错误的代码
        source_content.put_slice((create_tab(level + 2) + format!("reply(Err(NativeObjectValue::Str(\"Invalid type of {}th parameter\".to_string())));\n", index).as_str()).as_bytes());
    } else {
        //生成同步返回错误的代码
        source_content.put_slice((create_tab(level + 2) + format!("return Some(Err(\"Invalid type of {}th parameter\".to_string()));\n", index).as_str()).as_bytes());
    }
    source_content.put_slice((create_tab(level + 1) + "},\n").as_bytes());
    source_content.put_slice((create_tab(level) + "}\n").as_bytes());

    Ok(())
}

//生成函数调用代码的实参列表模式匹配子句
fn generate_function_call_args_match_cause(target: Option<&String>,
                                           generic: Option<&Generic>,
                                           function: &Function,
                                           args: &[(String, Type)],
                                           index: usize,
                                           level: isize,
                                           arg_names: &mut Vec<String>,
                                           source_content: &mut Vec<u8>,
                                           func_name: &String,
                                           origin_arg_name: &String,
                                           arg_name: &String,
                                           arg_type: &Type,
                                           arg_type_name: &TypeName,
                                           generic_type: &TypeName) -> Result<()> {
    match generic_type.get_name().as_str() {
        "bool" => {
            //生成匹配布尔值类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bool(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = *val;\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &*val;\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_ = *val;\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut val_;\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"i8" | alias@"i16" | alias@"i32" | alias@"i64" | alias@"i128" | alias@"isize" => {
            //生成匹配有符号整数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Int(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = (*val) as " + alias + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &((*val) as " + alias + ");\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_ = ((*val) as " + alias + ");\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut val_;\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"u8" | alias@"u16" | alias@"u32" | alias@"u64" | alias@"u128" | alias@"usize" => {
            //生成匹配无符号整数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Uint(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = (*val) as " + alias + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &((*val) as " + alias + ");\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_ = ((*val) as " + alias + ");\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut val_;\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"f32" | alias@"f64" => {
            //生成匹配浮点数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Float(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = (*val) as " + alias + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &((*val) as " + alias + ");\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_ = ((*val) as " + alias + ");\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut val_;\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "str" => {
            //生成匹配字符串类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Str(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                return Err(Error::new(ErrorKind::Other, format!("Generate function call args failed, function: {}, arg: {}, reason: not allowed take owner of str type", func_name, origin_arg_name)));
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = val.as_str();\n").as_bytes());
            } else if arg_type_name.is_writable() {
                return Err(Error::new(ErrorKind::Other, format!("Generate function call args failed, function: {}, arg: {}, reason: not allowed take mutable borrow of str type", func_name, origin_arg_name)))
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "String" => {
            //生成匹配字符串类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Str(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = val.clone();\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = val;\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_ = val.clone();\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut val_;\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "[u8]" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                return Err(Error::new(ErrorKind::Other, format!("Generate function call args failed, function: {}, arg: {}, reason: not allowed take owner of [u8] type", func_name, origin_arg_name)));
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = val.bytes();\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = val.bytes_mut();\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Arc<[u8]>" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + ": Arc<[u8]> = Arc::from(val.bytes());\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let val_: Arc<[u8]> = Arc::from(val.bytes());\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &val_;\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_: Arc<[u8]> = Arc::from(val.bytes());\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut val_;\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Box<[u8]>" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + ": Box<[u8]> = Box::new(val.bytes().to_vec()).into_boxed_slice();\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let val_: Box<[u8]> = Box::new(val.bytes().to_vec()).into_boxed_slice();\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &val_;\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_: Box<[u8]> = Box::new(val.bytes().to_vec()).into_boxed_slice();\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut val_;\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Arc<Vec<u8>>" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = Arc::new(val.bytes().to_vec());\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let val_ = Arc::new(val.bytes().to_vec());\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &val_;\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_ = Arc::new(val.bytes().to_vec());\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut val_;\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Box<Vec<u8>>" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = Box::new(val.bytes().to_vec());\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let val_ = Box::new(val.bytes().to_vec());\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &val_;\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_ = Box::new(val.bytes().to_vec());\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut val_;\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Vec<u8>" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = val.bytes().to_vec();\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let val_ = val.bytes().to_vec();\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &val_;\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_ = val.bytes().to_vec();\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut val_;\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        other_type => {
            //生成匹配其它类型的只读引用的代码，例: &NativeObject
            source_content.put_slice((create_tab(level) + "NativeObjectValue::NatObj(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                return Err(Error::new(ErrorKind::Other, format!("Generate function call args failed, function: {}, arg: {}, reason: not allowed take owner of {} type", func_name, origin_arg_name, other_type)));
            } else if arg_type_name.is_only_read() {
                let real_other_type = if arg_type.get_type_args().is_some() {
                    arg_type.to_string()
                } else {
                    other_type.to_string()
                };

                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + "_arc = val.get_ref::<" + real_other_type.as_str() + ">().unwrap().upgrade().unwrap();\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = " + arg_name.as_str() + "_arc.as_ref();\n").as_bytes());
            } else if arg_type_name.is_writable() {
                let real_other_type = if arg_type.get_type_args().is_some() {
                    arg_type.to_string()
                } else {
                    other_type.to_string()
                };

                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = val.get_mut::<" + real_other_type.as_str() + ">().unwrap();\n").as_bytes());
            }

            let next_index = index + 1;
            if next_index == args.len() {
                //实参列表已生成完成，则生成函数调用代码
                if let Err(e) = generate_call_function(target, generic, function, level + 1, arg_names, source_content, func_name) {
                    return Err(e);
                }
            } else {
                //否则继续生成下一个参数的代码
                if let Err(e) = generate_function_call_args(target, generic, function, args, next_index, level + 1, arg_names, source_content) {
                    return Err(e);
                }
            }

            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        }
    }

    Ok(())
}

//生成调用函数的代码
fn generate_call_function(target: Option<&String>,
                          generic: Option<&Generic>,
                          function: &Function,
                          level: isize,
                          arg_names: &mut Vec<String>,
                          source_content: &mut Vec<u8>,
                          func_name: &String) -> Result<()> {
    if let Some(target_name) = target {
        //指定目录的函数
        if function.is_static() {
            //关联函数
            if function.is_async() {
                //生成调用指定目标的异步方法代码
                let mut iterator = arg_names.iter();

                source_content.put_slice((create_tab(level) + "let result = " + get_target_type_name(target_name).as_str() + "::" + func_name + "(").as_bytes());
                if arg_names.len() > 0 {
                    //调用方法有参数
                    let arg_1 = iterator.next().unwrap();
                    source_content.put_slice(arg_1.as_bytes());

                    while let Some(func_arg) = iterator.next() {
                        source_content.put_slice((", ".to_string() + func_arg).as_bytes());
                    }
                }
                if function.is_async() {
                    //生成异步调用代码
                    source_content.put_slice(b").await;\n");
                } else {
                    //生成同步调用代码
                    source_content.put_slice(b");\n");
                }
            } else {
                //生成调用指定目录的同步方法代码
                let mut iterator = arg_names.iter();

                source_content.put_slice((create_tab(level) + "let result = " + get_target_type_name(target_name).as_str() + "::" + func_name + "(").as_bytes());
                if arg_names.len() > 0 {
                    //调用方法有参数
                    let arg_1 = iterator.next().unwrap();
                    source_content.put_slice(arg_1.as_bytes());

                    while let Some(func_arg) = iterator.next() {
                        source_content.put_slice((", ".to_string() + func_arg).as_bytes());
                    }
                }
                if function.is_async() {
                    //生成异步调用代码
                    source_content.put_slice(b").await;\n");
                } else {
                    //生成同步调用代码
                    source_content.put_slice(b");\n");
                }
            }
        } else {
            //方法
            if function.is_async() {
                //生成调用指定目标的异步方法代码
                let mut iterator = arg_names.iter();

                source_content.put_slice((create_tab(level) + "let result = self_obj." + func_name + "(").as_bytes());
                if arg_names.len() > 0 {
                    //调用方法有参数
                    let arg_1 = iterator.next().unwrap();
                    source_content.put_slice(arg_1.as_bytes());

                    while let Some(func_arg) = iterator.next() {
                        source_content.put_slice((", ".to_string() + func_arg).as_bytes());
                    }
                }
                if function.is_async() {
                    //生成异步调用代码
                    source_content.put_slice(b").await;\n");
                } else {
                    //生成同步调用代码
                    source_content.put_slice(b");\n");
                }
            } else {
                //生成调用指定目录的同步方法代码
                let mut iterator = arg_names.iter();

                source_content.put_slice((create_tab(level) + "let result = self_obj." + func_name + "(").as_bytes());
                if arg_names.len() > 0 {
                    //调用方法有参数
                    let arg_1 = iterator.next().unwrap();
                    source_content.put_slice(arg_1.as_bytes());

                    while let Some(func_arg) = iterator.next() {
                        source_content.put_slice((", ".to_string() + func_arg).as_bytes());
                    }
                }
                if function.is_async() {
                    //生成异步调用代码
                    source_content.put_slice(b").await;\n");
                } else {
                    //生成同步调用代码
                    source_content.put_slice(b");\n");
                }
            }
        }
    } else {
        //静态函数
        if function.is_async() {
            //生成调用异步静态函数代码
            source_content.put_slice((create_tab(level) + "let result = " + func_name + "(").as_bytes());
            if arg_names.len() > 0 {
                //调用静态函数有参数
                let mut iterator = arg_names.iter();
                let arg_0 = iterator.next().unwrap();
                source_content.put_slice(arg_0.as_bytes());

                while let Some(func_arg) = iterator.next() {
                    source_content.put_slice((", ".to_string() + func_arg).as_bytes());
                }
            }
            if function.is_async() {
                //生成异步调用代码
                source_content.put_slice(b").await;\n");
            } else {
                //生成同步调用代码
                source_content.put_slice(b");\n");
            }
        } else {
            //生成调用同步静态函数代码
            source_content.put_slice((create_tab(level) + "let result = " + func_name + "(").as_bytes());
            if arg_names.len() > 0 {
                //调用静态函数有参数
                let mut iterator = arg_names.iter();
                let arg_0 = iterator.next().unwrap();
                source_content.put_slice(arg_0.as_bytes());

                while let Some(func_arg) = iterator.next() {
                    source_content.put_slice((", ".to_string() + func_arg).as_bytes());
                }
            }
            if function.is_async() {
                //生成异步调用代码
                source_content.put_slice(b").await;\n");
            } else {
                //生成同步调用代码
                source_content.put_slice(b");\n");
            }
        }
    }

    generate_function_call_result(target, generic, function, level, source_content, func_name)
}

//生成函数调用的返回值
fn generate_function_call_result(target: Option<&String>,
                                 generic: Option<&Generic>,
                                 function: &Function,
                                 level: isize,
                                 source_content: &mut Vec<u8>,
                                 func_name: &String) -> Result<()> {
    match function.get_output() {
        None => {
            //函数调用没有返回值
            if function.is_async() {
                //生成异步返回空值的代码
                source_content.put_slice((create_tab(level) + "reply(Ok(NativeObjectValue::empty()));\n").as_bytes());
            } else {
                //生成同步返回空值的代码
                source_content.put_slice((create_tab(level) + "return None;\n").as_bytes());
            }
            return Ok(());
        },
        Some(return_type) => {
            //函数调用有返回值
            match return_type.get_part_type_name() {
                TypeName::Moveable(return_part_type_name) => {
                    //返回值为所有权
                    match return_part_type_name.as_str() {
                        "Option" => {
                            //返回值为Option
                            source_content.put_slice((create_tab(level) + "if let Some(r) = result {\n").as_bytes());

                            let return_type_arg_names = return_type.get_type_arg_names().unwrap();
                            if let Err(e) = generate_function_call_result_type(target, generic, function, level, source_content, func_name, &return_type_arg_names[0]) {
                                return Err(e);
                            }

                            source_content.put_slice((create_tab(level) + "} else {\n").as_bytes());
                            if function.is_async() {
                                //生成异步返回空值的代码
                                source_content.put_slice((create_tab(level + 1) + "reply(Ok(NativeObjectValue::empty()));\n").as_bytes());
                            } else {
                                //生成同步返回空值的代码
                                source_content.put_slice((create_tab(level + 1) + "return None;\n").as_bytes());
                            }
                            source_content.put_slice((create_tab(level) + "}\n").as_bytes());
                        },
                        "Result" => {
                            //返回值为Reuslt
                            source_content.put_slice((create_tab(level) + "match result {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 1) + "Err(e) => {\n").as_bytes());
                            if function.is_async() {
                                //生成异步返回错误的代码
                                source_content.put_slice((create_tab(level + 2) + "reply(Err(NativeObjectValue::Str(format!(\"{:?}\", e))));\n").as_bytes());
                            } else {
                                //生成同步返回错误的代码
                                source_content.put_slice((create_tab(level + 2) + "return Some(Err(format!(\"{:?}\", e)));\n").as_bytes());
                            }
                            source_content.put_slice((create_tab(level + 1) + "},\n").as_bytes());
                            source_content.put_slice((create_tab(level + 1) + "Ok(r) => {\n").as_bytes());

                            let return_type_arg_names = return_type.get_type_arg_names().unwrap();
                            if let Err(e) = generate_function_call_result_type(target, generic, function, level + 1, source_content, func_name, &return_type_arg_names[0]) {
                                return Err(e);
                            }

                            source_content.put_slice((create_tab(level + 1) + "},\n").as_bytes());
                            source_content.put_slice((create_tab(level) + "}\n").as_bytes());
                        },
                        _ => {
                            //其它返回值
                            source_content.put_slice((create_tab(level) + "let r = result;\n").as_bytes());

                            let return_type_name = return_type.get_type_name();
                            if let Err(e) = generate_function_call_result_type(target, generic, function, level - 1, source_content, func_name, &return_type_name) {
                                return Err(e);
                            }
                        },
                    }
                },
                TypeName::OnlyRead(return_part_type_name) => {
                    //返回值为只读引用
                    match return_part_type_name.as_str() {
                        "Option" => {
                            //返回值为Option
                            return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take only read borrow of Option type", func_name)));
                        },
                        "Result" => {
                            //返回值为Reuslt
                            return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take only read borrow of Result type", func_name)));
                        },
                        _ => {
                            //其它返回值
                            source_content.put_slice((create_tab(level) + "let r = result;\n").as_bytes());

                            let return_type_name = return_type.get_type_name();
                            if let Err(e) = generate_function_call_result_type(target, generic, function, level - 1, source_content, func_name, &return_type_name) {
                                return Err(e);
                            }
                        },
                    }
                },
                TypeName::Writable(return_part_type_name) => {
                    //返回值为可写引用
                    match return_part_type_name.as_str() {
                        "Option" => {
                            //返回值为Option
                            return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Option type", func_name)));
                        },
                        "Result" => {
                            //返回值为Reuslt
                            return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Result type", func_name)));
                        },
                        _ => {
                            //其它返回值
                            source_content.put_slice((create_tab(level) + "let r = result;\n").as_bytes());

                            let return_type_name = return_type.get_type_name();
                            if let Err(e) = generate_function_call_result_type(target, generic, function, level - 1, source_content, func_name, &return_type_name) {
                                return Err(e);
                            }
                        },
                    }
                },
            }
        },
    }

    Ok(())
}

//生成函数调用返回值的类型
fn generate_function_call_result_type(target: Option<&String>,
                                      generic: Option<&Generic>,
                                      function: &Function,
                                      level: isize,
                                      source_content: &mut Vec<u8>,
                                      func_name: &String,
                                      return_type: &TypeName) -> Result<()> {
    //首先在函数声明的泛型参数中检查返回值的类型
    let func_generic = function.get_generic();
    if let Some(func_generic) = func_generic {
        for (generic_name, generic_types) in func_generic.get_ref() {
            if return_type.get_name() == generic_name {
                //返回值是函数声明的泛型，则生成匹配开始代码
                source_content.put_slice((create_tab(level + 1) + "match r {\n").as_bytes());

                //获取函数声明的泛型的具体类型，并生成匹配具体类型的代码
                for generic_type in generic_types {
                    if let Err(e) = generate_function_call_result_match_cause(target, generic, function, level + 1, source_content, func_name, return_type, Some(generic_type)) {
                        return Err(e);
                    }
                }

                //生成剩余匹配项和匹配结束的代码
                source_content.put_slice((create_tab(level + 2) + "_ => {\n").as_bytes());
                if function.is_async() {
                    //生成异步返回错误的代码
                    source_content.put_slice((create_tab(level + 3) + "reply(Err(NativeObjectValue::Str(\"Invalid return type\".to_string())));\n").as_bytes());
                } else {
                    //生成同步返回错误的代码
                    source_content.put_slice((create_tab(level + 3) + "return Some(Err(\"Invalid return type\".to_string()));\n").as_bytes());
                }
                source_content.put_slice((create_tab(level + 2) + "},\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "}\n").as_bytes());

                return Ok(());
            }
        }
    }

    //然后在对象声明的泛型参数中检查参数类型
    if let Some(generic_) = generic {
        for (generic_name, generic_types) in generic_.get_ref() {
            if return_type.get_name() == generic_name {
                //返回值是函数声明的泛型，则生成匹配开始代码
                source_content.put_slice((create_tab(level + 1) + "match r {\n").as_bytes());

                //获取函数声明的泛型的具体类型，并生成匹配具体类型的代码
                for generic_type in generic_types {
                    if let Err(e) = generate_function_call_result_match_cause(target, generic, function, level + 1, source_content, func_name, return_type, Some(generic_type)) {
                        return Err(e);
                    }
                }

                //生成剩余匹配项和匹配结束的代码
                source_content.put_slice((create_tab(level + 2) + "_ => {\n").as_bytes());
                if function.is_async() {
                    //生成异步返回错误的代码
                    source_content.put_slice((create_tab(level + 3) + "reply(Err(NativeObjectValue::Str(\"Invalid return type\".to_string())));\n").as_bytes());
                } else {
                    //生成同步返回错误的代码
                    source_content.put_slice((create_tab(level + 3) + "return Some(Err(\"Invalid return type\".to_string()));\n").as_bytes());
                }
                source_content.put_slice((create_tab(level + 2) + "},\n").as_bytes());
                source_content.put_slice((create_tab(level + 1) + "}\n").as_bytes());

                return Ok(());
            }
        }
    }

    //返回值类型是具体类型
    generate_function_call_result_match_cause(target, generic, function, level + 1, source_content, func_name, return_type, None)
}

//生成函数调用返回值的模式匹配子句
fn generate_function_call_result_match_cause(target: Option<&String>,
                                             generic: Option<&Generic>,
                                             function: &Function,
                                             level: isize,
                                             source_content: &mut Vec<u8>,
                                             func_name: &String,
                                             return_type: &TypeName,
                                             generic_type: Option<&TypeName>) -> Result<()> {
    match return_type.get_name().as_str() {
        "bool" => {
            //生成匹配布尔值类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<bool>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bool(r)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bool(*r)));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bool(*r)));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bool(r)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bool(*r)));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bool(*r)));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"i8" | alias@"i16" | alias@"i32" | alias@"i64" | alias@"i128" | alias@"isize" => {
            //生成匹配有符号整数类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<i8>() || r.is::<i16>() || r.is::<i32>() || r.is::<i64>() || r.is::<i128>() || r.is::<isize>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Int(r as i32)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Int((*r) as i32)));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Int((*r) as i32)));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Int(r as i32)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Int((*r) as i32)));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Int((*r) as i32)));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"u8" | alias@"u16" | alias@"u32" | alias@"u64" | alias@"u128" | alias@"usize" => {
            //生成匹配无符号整数类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<u8>() || r.is::<u16>() || r.is::<u32>() || r.is::<u64>() || r.is::<u128>() || r.is::<usize>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Uint(r as u32)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Uint((*r) as u32)));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Uint((*r) as u32)));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Uint(r as u32)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Uint((*r) as u32)));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Uint((*r) as u32)));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"f32" | alias@"f64" => {
            //生成匹配浮点数类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<f32>() || r.is::<f64>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Float(r as f64)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Float((*r) as f64)));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Float((*r) as f64)));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Float(r as f64)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Float((*r) as f64)));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Float((*r) as f64)));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "str" => {
            //生成匹配字符串类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<&str>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take owner of str type", func_name)));
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Str(r.to_string())));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Str(r.to_string())));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take owner of str type", func_name)));
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Str(r.to_string())));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Str(r.to_string())));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "String" => {
            //生成匹配字符串类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<String>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Str(r)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Str(r.clone())));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Str(r.clone())));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Str(r)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Str(r.clone())));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Str(r.clone())));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "[u8]" => {
            //生成匹配二进制缓冲区类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<&[u8]>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take owner of [u8] type", func_name)));
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take owner of [u8] type", func_name)));
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Arc<[u8]>" => {
            //生成匹配二进制缓冲区类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<Arc<[u8]>>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Box<[u8]>" => {
            //生成匹配二进制缓冲区类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<Box<[u8]>>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Arc<Vec<u8>>" => {
            //生成匹配二进制缓冲区类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<Arc<Vec<u8>>>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Box<Vec<u8>>" => {
            //生成匹配二进制缓冲区类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<Box<Vec<u8>>>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(*r))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(*r))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<u8>" => {
            //生成匹配二进制缓冲区类型的代码
            let mut current_level = level;
            if let Some(generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<Vec<u8>>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        other_type => {
            //生成匹配其它类型的只读引用的代码，例: &NativeObject
            if let Some(generic_type) = generic_type {
                //泛型的具体类型
                if let Err(e) = generate_function_call_result_match_cause(target, generic, function, level + 1, source_content, func_name, generic_type, Some(generic_type)) {
                    return Err(e);
                }
            } else {
                if function.is_async() {
                    //生成异步返回代码
                    if return_type.is_moveable() {
                        source_content.put_slice((create_tab(level) + "reply(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));\n").as_bytes());
                    } else if return_type.is_only_read() {
                        return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take only read borrow of {} type", func_name, other_type)));
                    } else if return_type.is_writable() {
                        return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of {} type", func_name, other_type)));
                    }
                } else {
                    //生成同步返回代码
                    if return_type.is_moveable() {
                        source_content.put_slice((create_tab(level) + "return Some(Ok(NativeObjectValue::NatObj(NativeObject::new_owned(r))));\n").as_bytes());
                    } else if return_type.is_only_read() {
                        return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take only read borrow of {} type", func_name, other_type)));
                    } else if return_type.is_writable() {
                        return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of {} type", func_name, other_type)));
                    }
                }
            }
        }
    }

    Ok(())
}