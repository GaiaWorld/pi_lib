use std::path::{Path, PathBuf, Component};
use std::io::{Error, Result, ErrorKind};

use bytes::BufMut;

use pi_async_file::file::{AsyncFileOptions, AsyncFile};

use crate::{WORKER_RUNTIME,
            utils::{ParseContext, ExportItem, Function, Generic, Type, TypeName, ProxySourceGenerater, ClosureType, create_tab, create_tmp_var_name, get_target_type_name, get_specific_ts_function_name}};

/*
* 默认的依赖库名
*/
#[cfg(target_os = "windows")]
pub(crate) const DEFAULT_DEPEND_CRATE_NAME: &str = r#"pi_v8\vm_builtin"#;
#[cfg(target_os = "linux")]
pub(crate) const DEFAULT_DEPEND_CRATE_NAME: &str = "pi_v8/vm_builtin";

/*
* 默认代理入口文件导入的类型
*/
pub(crate) const DEFAULT_PROXY_LIB_FILE_USED: &[u8] = b"use vm_builtin::external::{register_native_object_static_method,\n\t\t\t\t\t\t\tregister_native_object_async_static_method,\n\t\t\t\t\t\t\tregister_native_object_method,\n\t\t\t\t\t\t\tregister_native_object_async_method};\n\n";

/*
* 默认的代理入口文件注册代理函数的函数签名
*/
pub(crate) const DEFAULT_PROXY_LIB_REGISTER_FUNCTION_NAME: &str = "/**\n * 注册所有自动导入的外部扩展库中声明的导出函数\n */\npub fn register_ext_functions() {\n";

/*
* 默认代理Rust文件导入的类型
*/
pub(crate) const DEFAULT_PROXY_RUST_FILE_USED: &[u8] = b"use std::any::Any;\nuse std::sync::Arc;\n\nuse futures::future::{FutureExt, BoxFuture};\nuse num_bigint::{ToBigInt, BigInt};\nuse num_traits::cast::{FromPrimitive, ToPrimitive};\n\nuse vm_builtin::{buffer::NativeArrayBuffer, external::{NativeObjectAsyncTaskSpawner, NativeObjectAsyncReply, NativeObjectValue, NativeObjectArgs, NativeObject}};\n\n";

/*
* 默认的代理函数签名前缀
*/
pub(crate) const DEFAULT_PROXY_FUNCTION_SING_PREFIX: &[u8] = b"pub fn ";

/*
* 默认的静态代理函数名前缀
*/
pub(crate) const DEFAULT_STATIC_PROXY_FUNCTION_NAME_PREFIX: &[u8] = b"static_call_";

/*
* 默认的异步静态代理函数名前缀
*/
pub(crate) const DEFAULT_ASYNC_STATIC_PROXY_FUNCTION_NAME_PREFIX: &[u8] = b"async_static_call_";

/*
* 默认的代理函数名前缀
*/
pub(crate) const DEFAULT_PROXY_FUNCTION_NAME_PREFIX: &[u8] = b"call_";

/*
* 默认的静态代理函数名前缀
*/
pub(crate) const DEFAULT_ASYNC_PROXY_FUNCTION_NAME_PREFIX: &[u8] = b"async_call_";

/*
* 默认的静态代理函数签名后缀
*/
pub(crate) const DEFAULT_STATIC_PROXY_FUNCTION_SIGN_SUFFIX: &[u8] = b"(args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {\n";

/*
* 默认的异步静态代理函数签名后缀
*/
pub(crate) const DEFAULT_ASYNC_STATIC_PROXY_FUNCTION_SIGN_SUFFIX: &[u8] = b"(args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {\n";

/*
* 默认的代理函数签名后缀
*/
pub(crate) const DEFAULT_PROXY_FUNCTION_SIGN_SUFFIX: &[u8] = b"(obj: &NativeObject, args: NativeObjectArgs) -> Option<Result<NativeObjectValue, String>> {\n";

/*
* 默认的静态代理函数签名后缀
*/
pub(crate) const DEFAULT_ASYNC_PROXY_FUNCTION_SIGN_SUFFIX: &[u8] = b"(obj: NativeObject, args: NativeObjectArgs, spawner: NativeObjectAsyncTaskSpawner, reply: NativeObjectAsyncReply) {\n";

/*
* 默认的代理函数块结束符
*/
pub(crate) const DEFAULT_PROXY_FUNCTION_BLOCK_END: &[u8] = b"}\n\n";

/*
* 默认的实参名称前缀
*/
pub(crate) const DEFAULT_ARGUMENT_NAME_PREFIX: &str = "arg_";

//在指定路径下创建代理的Rust文件，并返回异步文件句柄
pub(crate) async fn create_proxy_rust_file(generater: &ProxySourceGenerater,
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

//生成Rust文件的导入
pub(crate) fn generate_rust_import(mut path_buf: PathBuf,
                                   _source: &ParseContext) -> Vec<u8> {
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
pub(crate) async fn generate_rust_functions(generater: &ProxySourceGenerater,
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
            //生成异步静态函数，则记录异步静态代理函数名，并分配异步静态代理函数序号
            let index =
                generater
                    .append_async_static_method(target, get_specific_ts_function_name(function), String::from_utf8(DEFAULT_ASYNC_STATIC_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap()).await;
            generate_async_static_function(target, generic, function, index, source_content)
        } else {
            //生成静态函数，则记录静态代理函数名，并分配静态代理函数序号
            let index =
                generater
                    .append_static_method(target, get_specific_ts_function_name(function), String::from_utf8(DEFAULT_STATIC_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap()).await;
            generate_static_function(target, generic, function, index, source_content)
        }
    } else {
        if function.is_async() {
            //生成异步函数，则记录异步代理函数名，并分配异步代理函数序号
            let index =
                generater
                    .append_async_method(target, get_specific_ts_function_name(function), String::from_utf8(DEFAULT_ASYNC_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap()).await;
            generate_async_function(target, generic, function, index, source_content)
        } else {
            //生成函数，则记录代理函数名，并分配代理函数序号
            let index =
                generater
                    .append_method(target, get_specific_ts_function_name(function), String::from_utf8(DEFAULT_PROXY_FUNCTION_NAME_PREFIX.to_vec()).unwrap()).await;
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
            let comment_str = " *".to_string() + comment.replace(r#"""#, "").replace(r#"\r"#, "").as_str() + "\n";
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
                        source_content.put_slice((create_tab(level) + "let self_obj = obj.get_ref::<" + target_name.as_str() + ">().unwrap();\n").as_bytes());
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
                    if let Err(e) = generate_function_call_args_match_cause_by_generic_type(target, generic, function, args, index, level + 1, arg_names, source_content, &func_name, origin_arg_name, &arg_name, &arg_type, arg_type_name, generic_type) {
                        return Err(e);
                    }
                    let _ = arg_names.pop(); //移除多余实参的名称
                }

                //生成剩余匹配项和匹配结束的代码
                source_content.put_slice((create_tab(level + 1) + "_ => {\n").as_bytes());
                if function.is_async() {
                    //生成异步返回错误的代码
                    source_content.put_slice((create_tab(level + 2) + format!("return reply(Err(NativeObjectValue::Str(\"Invalid type of {}th parameter\".to_string())));\n", index).as_str()).as_bytes());
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
                    if let Err(e) = generate_function_call_args_match_cause_by_generic_type(target, generic, function, args, index, level + 1, arg_names, source_content, &func_name, origin_arg_name, &arg_name, &arg_type, arg_type_name, generic_type) {
                        return Err(e);
                    }
                    let _ = arg_names.pop(); //移除多余实参的名称
                }

                //生成剩余匹配项和匹配结束的代码
                source_content.put_slice((create_tab(level + 1) + "_ => {\n").as_bytes());
                if function.is_async() {
                    //生成异步返回错误的代码
                    source_content.put_slice((create_tab(level + 2) + format!("return reply(Err(NativeObjectValue::Str(\"Invalid type of {}th parameter\".to_string())));\n", index).as_str()).as_bytes());
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
    source_content.put_slice((create_tab(level) + "let mut " + create_tmp_var_name(index).as_str() + " = match &args[" + index.to_string().as_str() + "] {\n").as_bytes());

    arg_names.push(arg_name.clone()); //记录指定实参的名称
    if let Err(e) = generate_function_call_args_match_cause(target, generic, function, args, index, level + 1, arg_names, source_content, &func_name, origin_arg_name, &arg_name, &arg_type, arg_type_name) {
        return Err(e);
    }

    //生成剩余匹配项和匹配结束的代码
    source_content.put_slice((create_tab(level + 1) + "_ => {\n").as_bytes());
    if function.is_async() {
        //生成异步返回错误的代码
        source_content.put_slice((create_tab(level + 2) + format!("return reply(Err(NativeObjectValue::Str(\"Invalid type of {}th parameter\".to_string())));\n", index).as_str()).as_bytes());
    } else {
        //生成同步返回错误的代码
        source_content.put_slice((create_tab(level + 2) + format!("return Some(Err(\"Invalid type of {}th parameter\".to_string()));\n", index).as_str()).as_bytes());
    }
    source_content.put_slice((create_tab(level + 1) + "},\n").as_bytes());
    source_content.put_slice((create_tab(level) + "};\n").as_bytes());

    //生成匹配布尔值类型的代码
    match arg_type_name.get_name().as_str() {
        "bool" => {
            //生成将参数转换为布尔值类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = *" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &*" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut *" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        alias@"i8" | alias@"i16" | alias@"i32" => {
            //生成将参数转换为符号整数类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        alias@"i64" | alias@"i128" | alias@"isize" => {
            //生成将参数转换为符号64位或128位整数类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ".to_i128().expect(\"From js bigint to " + alias + " failed\") as " + alias + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &(" + create_tmp_var_name(index).as_str() + ".to_i128().expect(\"From js bigint to " + alias + " ref failed\")) as " + alias + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut (" + create_tmp_var_name(index).as_str() + ".to_i128().expect(\"From js bigint to " + alias + " mut ref failed\") as " + alias + ");\n\n").as_bytes());
            }
        },
        alias@"u8" | alias@"u16" | alias@"u32" => {
            //生成将参数转换为无符号整数类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        alias@"u64" | alias@"u128" | alias@"usize" => {
            //生成将参数转换为无符号64位或128位整数类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ".to_u128().expect(\"From js bigint to " + alias + " failed\") as " + alias + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &(" + create_tmp_var_name(index).as_str() + ".to_u128().expect(\"From js bigint to " + alias + " ref failed\") as " + alias + ");\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ".to_u128().expect(\"From js bigint to " + alias + " mut ref failed\") as " + alias + ");;\n\n").as_bytes());
            }
        },
        alias@"f32" | alias@"f64" => {
            //生成将参数转换为浮点数类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        "BigInt" => {
            //生成将参数转换为有符号大整数类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        "str" => {
            //生成将参数转换为字符串类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                return Err(Error::new(ErrorKind::Other, format!("Generate function call args failed, function: {}, arg: {}, reason: not allowed take owner of str type", func_name, origin_arg_name)));
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ".as_str();\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                return Err(Error::new(ErrorKind::Other, format!("Generate function call args failed, function: {}, arg: {}, reason: not allowed take mutable borrow of str type", func_name, origin_arg_name)))
            }
        },
        "String" => {
            //生成将参数转换为字符串类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        "[u8]" => {
            //生成将参数转换为二进制缓冲区类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                return Err(Error::new(ErrorKind::Other, format!("Generate function call args failed, function: {}, arg: {}, reason: not allowed take owner of [u8] type", func_name, origin_arg_name)));
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ".bytes();\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ".bytes_mut();\n\n").as_bytes());
            }
        },
        "Arc<[u8]>" => {
            //生成将参数转换为二进制缓冲区类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + ": Arc<[u8]> = Arc::from(" + create_tmp_var_name(index).as_str() + ".bytes());\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + ": &Arc<[u8]> = &Arc::from(" + create_tmp_var_name(index).as_str() + ".bytes());\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + ": &mut Arc<[u8]> = &mut Arc::from(" + create_tmp_var_name(index).as_str() + ".bytes());\n\n").as_bytes());
            }
        },
        "Box<[u8]>" => {
            //生成将参数转换为二进制缓冲区类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = Box::new(" + create_tmp_var_name(index).as_str() + ".bytes().to_vec()).into_boxed_slice();\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &Box::new(" + create_tmp_var_name(index).as_str() + ".bytes().to_vec()).into_boxed_slice();\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut Box::new(" + create_tmp_var_name(index).as_str() + ".bytes().to_vec()).into_boxed_slice();\n\n").as_bytes());
            }
        },
        "Arc<Vec<u8>>" => {
            //生成将参数转换为二进制缓冲区类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = Arc::new(" + create_tmp_var_name(index).as_str() + ".bytes().to_vec());\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &Arc::new(" + create_tmp_var_name(index).as_str() + ".bytes().to_vec());\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut Arc::new(" + create_tmp_var_name(index).as_str() + ".bytes().to_vec());\n\n").as_bytes());
            }
        },
        "Box<Vec<u8>>" => {
            //生成将参数转换为二进制缓冲区类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = Box::new(" + create_tmp_var_name(index).as_str() + ".bytes().to_vec());\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &Box::new(" + create_tmp_var_name(index).as_str() + ".bytes().to_vec());\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut Box::new(" + create_tmp_var_name(index).as_str() + ".bytes().to_vec());\n\n").as_bytes());
            }
        },
        "Vec<u8>" => {
            //生成将参数转换为二进制缓冲区类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ".bytes().to_vec();\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ".bytes().to_vec();;\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ".bytes().to_vec();\n\n").as_bytes());
            }
        },
        "Vec<bool>" => {
            //生成将参数转换为布尔值数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        alias@"Vec<i8>" | alias@"Vec<i16>" | alias@"Vec<i32>" => {
            //生成将参数转换为有符号整数数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        alias@"Vec<i64>" | alias@"Vec<i128>" | alias@"Vec<isize>" => {
            //生成将参数转换为有符号64位或128位整数数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        alias@"Vec<u16>" | alias@"Vec<u32>" => {
            //生成将参数转换为无符号整数数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        alias@"Vec<u64>" | alias@"Vec<u128>" | alias@"Vec<usize>" => {
            //生成将参数转换为无符号64位或128位整数数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        alias@"Vec<f32>" | alias@"Vec<f64>" => {
            //生成将参数转换为浮点数数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        alias@"Vec<BigInt>" => {
            //生成将参数转换为无符号大整数数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        "Vec<String>" => {
            //生成将参数转换为字符串数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        "Vec<Arc<[u8]>>" => {
            //生成将参数转换为二进制缓冲区数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        "Vec<Box<[u8]>>" => {
            //生成将参数转换为二进制缓冲区数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        "Vec<Arc<Vec<u8>>>" => {
            //生成将参数转换为二进制缓冲区数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        "Vec<Box<Vec<u8>>>" => {
            //生成将参数转换为二进制缓冲区数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        "Vec<Vec<u8>>" => {
            //生成将参数转换为二进制缓冲区数组类型和指定所有权的代码
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &" + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = &mut " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            }
        },
        closure_type if closure_type.starts_with("Arc<Fn(") => {
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ";\n\n").as_bytes());
            } else {
                return Err(Error::new(ErrorKind::Other, format!("Generate function call args failed, function: {}, arg: {}, reason: not allowed take borrow of {} type", func_name, origin_arg_name, closure_type)));
            }
        },
        other_type => {
            //生成将参数转换为其它类型的只读引用和指定所有权的代码，例: &NativeObject
            if arg_type_name.is_moveable() {
                return Err(Error::new(ErrorKind::Other, format!("Generate function call args failed, function: {}, arg: {}, reason: not allowed take owner of {} type", func_name, origin_arg_name, other_type)));
            } else if arg_type_name.is_only_read() {
                let real_other_type = if arg_type.get_type_args().is_some() {
                    arg_type.to_string()
                } else {
                    other_type.to_string()
                };

                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ".get_ref::<" + real_other_type.as_str() + ">().unwrap();\n\n").as_bytes());
            } else if arg_type_name.is_writable() {
                let real_other_type = if arg_type.get_type_args().is_some() {
                    arg_type.to_string()
                } else {
                    other_type.to_string()
                };

                source_content.put_slice((create_tab(level) + "let " + arg_name.as_str() + " = " + create_tmp_var_name(index).as_str() + ".get_mut::<" + real_other_type.as_str() + ">().unwrap();\n\n").as_bytes());
            }
        },
    }

    let next_index = index + 1;
    if next_index == args.len() {
        //实参列表已生成完成，则生成函数调用代码
        if let Err(e) = generate_call_function(target,
                                               generic,
                                               function,
                                               level,
                                               arg_names,
                                               source_content,
                                               &func_name) {
            return Err(e);
        }
    } else {
        //否则继续生成下一个参数的代码
        if let Err(e) = generate_function_call_args(target,
                                                    generic,
                                                    function,
                                                    args,
                                                    next_index,
                                                    level,
                                                    arg_names,
                                                    source_content) {
            return Err(e);
        }
    }
    let _ = arg_names.pop(); //移除多余实参的名称

    Ok(())
}

//生成函数调用代码的实参列表模式匹配子句，实参类型为泛型的某个具体类型
fn generate_function_call_args_match_cause_by_generic_type(target: Option<&String>,
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
        alias@"i8" | alias@"i16" | alias@"i32" => {
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
        alias@"i64" | alias@"i128" | alias@"isize" => {
            //生成匹配有符号64位或128位整数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::BigInt(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = val.to_i128().expect(\"From js bigint to " + alias + " failed\") as " + alias + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &(val.to_i128().expect(\"From js bigint to " + alias + " ref failed\")) as " + alias + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_ = (val.to_i128().expect(\"From js bigint to " + alias + " mut ref failed\") as " + alias + ");\n").as_bytes());
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
        alias@"u8" | alias@"u16" | alias@"u32" => {
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
        alias@"u64" | alias@"u128" | alias@"usize" => {
            //生成匹配无符号64位或128位整数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::BigInt(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = val.to_u128().expect(\"From js bigint to " + alias + " failed\") as " + alias + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &(val.to_u128().expect(\"From js bigint to " + alias + " ref failed\") as " + alias + ");\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let mut val_ = (val.to_u128().expect(\"From js bigint to " + alias + " mut ref failed\") as " + alias + ");\n").as_bytes());
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

            //生成匹配有符号整数类型的代码，当浮点数被强制转为有符号整数时进行匹配
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

            //生成匹配无符号整数类型的代码，当浮点数被强制转为无符号整数时进行匹配
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
        "BigInt" => {
            //生成匹配有符号大整数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::BigInt(val) => {\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = val.clone();\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &val;\n").as_bytes());
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
        "Vec<bool>" => {
            //生成匹配布尔值数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bool(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(*val);\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to bool failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        alias@"Vec<i8>" | alias@"Vec<i16>" | alias@"Vec<i32>" => {
            //生成匹配有符号整数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Int(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        alias@"Vec<i64>" | alias@"Vec<i128>" | alias@"Vec<isize>" => {
            //生成匹配有符号64位或128位整数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::BigInt(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(val.to_i128().expect(\"From js bigint array to " + alias + " failed\") as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        alias@"Vec<u16>" | alias@"Vec<u32>" => {
            //生成匹配无符号整数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Uint(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        alias@"Vec<u64>" | alias@"Vec<u128>" | alias@"Vec<usize>" => {
            //生成匹配无符号64位或128位整数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::BigInt(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(val.to_u128().expect(\"From js bigint array to " + alias + " failed\") as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        alias@"Vec<f32>" | alias@"Vec<f64>" => {
            //生成匹配浮点数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "match obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Float(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Int(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Uint(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "_ => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        alias@"Vec<BigInt>" => {
            //生成匹配无符号大整数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::BigInt(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(val.clone());\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        "Vec<String>" => {
            //生成匹配字符串数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Str(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(val.clone());\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to String failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        "Vec<Arc<[u8]>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": Vec<Arc<[u8]>> = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bin(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(Arc::from(val.bytes()));\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to Arc<[u8]> failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        "Vec<Box<[u8]>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": Vec<Box<[u8]>> = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bin(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(Box::new(val.bytes().to_vec()).into_boxed_slice());\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to Box<[u8]> failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        "Vec<Arc<Vec<u8>>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": Vec<Arc<Vec<u8>>> = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bin(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(Arc::new(val.bytes().to_vec()));\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to Arc<Vec<u8>> failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        "Vec<Box<Vec<u8>>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": Vec<Box<Vec<u8>>> = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bin(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(Box::new(val.bytes().to_vec()));\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to Box<Vec<u8>> failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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
        "Vec<Vec<u8>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": Vec<Vec<u8>> = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bin(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(val.bytes().to_vec());\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to Vec<u8> failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            if arg_type_name.is_moveable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_only_read() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &array_" + arg_name.as_str() + ";\n").as_bytes());
            } else if arg_type_name.is_writable() {
                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = &mut array_" + arg_name.as_str() + ";\n").as_bytes());
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

                source_content.put_slice((create_tab(level + 1) + "let " + arg_name.as_str() + " = val.get_ref::<" + real_other_type.as_str() + ">().unwrap();\n").as_bytes());
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
                                           generic_type: &TypeName) -> Result<()> {
    match generic_type.get_name().as_str() {
        "bool" => {
            //生成匹配布尔值类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bool(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"i8" | alias@"i16" | alias@"i32" => {
            //生成匹配有符号整数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Int(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "(*val) as " + alias + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());

            //生成匹配有符号整数类型的代码，当有符号整数被强制转为无符号整数时进行匹配
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Uint(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "(*val) as " + alias + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());

            //生成匹配有符号整数类型的代码，当有符号整数被强制转为浮点数时进行匹配
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Float(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "(*val) as " + alias + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"i64" | alias@"i128" | alias@"isize" => {
            //生成匹配有符号64位或128位整数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::BigInt(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"u8" | alias@"u16" | alias@"u32" => {
            //生成匹配无符号整数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Uint(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "(*val) as " + alias + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());

            //生成匹配无符号整数类型的代码，当无符号整数被强制转为有符号整数时进行匹配
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Int(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "(*val) as " + alias + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());

            //生成匹配无符号整数类型的代码，当无符号整数被强制转为浮点整数时进行匹配
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Float(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "(*val) as " + alias + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"u64" | alias@"u128" | alias@"usize" => {
            //生成匹配无符号64位或128位整数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::BigInt(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"f32" | alias@"f64" => {
            //生成匹配浮点数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Float(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "(*val) as " + alias + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());

            //生成匹配有符号整数类型的代码，当浮点数被强制转为有符号整数时进行匹配
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Int(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "(*val) as " + alias + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());

            //生成匹配无符号整数类型的代码，当浮点数被强制转为无符号整数时进行匹配
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Uint(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "(*val) as " + alias + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "BigInt" => {
            //生成匹配有符号大整数类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::BigInt(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "str" => {
            //生成匹配字符串类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Str(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "String" => {
            //生成匹配字符串类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Str(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val.clone()\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "[u8]" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Arc<[u8]>" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Box<[u8]>" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Arc<Vec<u8>>" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Box<Vec<u8>>" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Vec<u8>" => {
            //生成匹配二进制缓冲区类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Bin(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Vec<bool>" => {
            //生成匹配布尔值数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bool(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(*val);\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to bool failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"Vec<i8>" | alias@"Vec<i16>" | alias@"Vec<i32>" => {
            //生成匹配有符号整数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "match obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Int(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Uint(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Float(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "_ => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"Vec<i64>" | alias@"Vec<i128>" | alias@"Vec<isize>" => {
            //生成匹配有符号64位或128位整数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::BigInt(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(val.to_i128().expect(\"From js bigint array to " + alias + " failed\") as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"Vec<u16>" | alias@"Vec<u32>" => {
            //生成匹配无符号整数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "match obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Uint(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Int(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Float(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "_ => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"Vec<u64>" | alias@"Vec<u128>" | alias@"Vec<usize>" => {
            //生成匹配无符号64位或128位整数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::BigInt(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(val.to_u128().expect(\"From js bigint array to " + alias + " failed\") as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"Vec<f32>" | alias@"Vec<f64>" => {
            //生成匹配浮点数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "match obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Float(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Int(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "NativeObjectValue::Uint(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "array_" + arg_name.as_str() + ".push(*val as " + sub_type + ");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "_ => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 4) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "},\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        alias@"Vec<BigInt>" => {
            //生成匹配无符号大整数数组类型的代码
            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
            let sub_type = sub_types[1];

            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": " + alias + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::BigInt(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(val.clone());\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to " + sub_type + " failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Vec<String>" => {
            //生成匹配字符串数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + " = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Str(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(val.clone());\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to String failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Vec<Arc<[u8]>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": Vec<Arc<[u8]>> = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bin(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(Arc::from(val.bytes()));\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to Arc<[u8]> failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Vec<Box<[u8]>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": Vec<Box<[u8]>> = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bin(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(Box::new(val.bytes().to_vec()).into_boxed_slice());\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to Box<[u8]> failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Vec<Arc<Vec<u8>>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": Vec<Arc<Vec<u8>>> = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bin(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(Arc::new(val.bytes().to_vec()));\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to Arc<Vec<u8>> failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Vec<Box<Vec<u8>>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": Vec<Box<Vec<u8>>> = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bin(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(Box::new(val.bytes().to_vec()));\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to Box<Vec<u8>> failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        "Vec<Vec<u8>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            source_content.put_slice((create_tab(level) + "NativeObjectValue::Array(array) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let mut array_" + arg_name.as_str() + ": Vec<Vec<u8>> = Vec::with_capacity(array.len());\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "for obj in array {\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "if let NativeObjectValue::Bin(val) = obj {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "array_" + arg_name.as_str() + ".push(val.bytes().to_vec());\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "} else {\n").as_bytes());
            source_content.put_slice((create_tab(level + 3) + "panic!(\"Parse native object in array to Vec<u8> failed\");\n").as_bytes());
            source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "array_" + arg_name.as_str() + "\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        closure_type if closure_type.starts_with("Arc<Fn(") => {
            //生成匹配可重复调用的回调函数类型的代码
            let vec: Vec<&str> = closure_type.split("Arc<Fn(").collect();
            let vec: Vec<&str> = vec[1].split(",").collect();

            let (args_type, result) = match vec.len() {
                2 => {
                    //回调函数没有参数
                    let vec: Vec<&str> = vec[0].split("Option<Box<FnOnce(Result<").collect();
                    (vec![], vec[1].to_string())
                },
                len => {
                    //回调函数有参数
                    let mut args = Vec::with_capacity(len - 2);
                    for index in 0..len - 2 {
                        args.push(vec[index].trim().to_string());
                    }

                    let vec: Vec<&str> = vec[len - 2].split("Option<Box<FnOnce(Result<").collect();

                    (args, vec[1].to_string())
                },
            };

            source_content.put_slice((create_tab(level) + "NativeObjectValue::CallBack(cb) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "let callback = cb.clone();\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "Arc::new(\n").as_bytes());
            match generate_closure_call_args(level + 2, source_content, &args_type, &result) {
                Err(e) => {
                    return Err(e);
                },
                Ok(args_name) => {
                    source_content.put_slice((create_tab(level + 3) + "let mut args = Vec::with_capacity(" + args_name.len().to_string().as_str() + ");\n\n").as_bytes());

                    //生成将闭包参数类型转换为js类型的代码
                    let mut index = 0;
                    for arg_name in args_name {
                        match args_type[index].as_str() {
                            "bool" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Bool(" + arg_name.as_str() + "));\n").as_bytes());
                            },
                            "i8" | "i16" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Int(" + arg_name.as_str() + " as i32));\n").as_bytes());
                            },
                            "i32" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Int(" + arg_name.as_str() + "));\n").as_bytes());
                            },
                            "u8" | "u16" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Uint(" + arg_name.as_str() + " as u32));\n").as_bytes());
                            },
                            "u32" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Uint(" + arg_name.as_str() + "));\n").as_bytes());
                            },
                            "f32" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Float(" + arg_name.as_str() + " as f64));\n").as_bytes());
                            },
                            "f64" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Float(" + arg_name.as_str() + "));\n").as_bytes());
                            },
                            "i64" | "u64" | "i128" | "u128" | "isize" | "usize" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::BigInt(" + arg_name.as_str() + ".to_bigint().unwrap()));\n").as_bytes());
                            },
                            "BigInt" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::BigInt(" + arg_name.as_str() + "));\n").as_bytes());
                            },
                            "str" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Str(" + arg_name.as_str() + ".to_string()));\n").as_bytes());
                            },
                            "String" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Str(" + arg_name.as_str() + "));\n").as_bytes());
                            },
                            "[u8]" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Bin(NativeArrayBuffer::from(" + arg_name.as_str() + ".to_vec().into_boxed_slice())));\n").as_bytes());
                            },
                            "Arc<[u8]>" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Bin(NativeArrayBuffer::from(" + arg_name.as_str() + ".to_vec().into_boxed_slice())));\n").as_bytes());
                            },
                            "Box<[u8]>" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Bin(NativeArrayBuffer::from(" + arg_name.as_str() + ")));\n").as_bytes());
                            },
                            "Arc<Vec<u8>>" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Bin(NativeArrayBuffer::from(" + arg_name.as_str() + ".to_vec().into_boxed_slice())));\n").as_bytes());
                            },
                            "Box<Vec<u8>>" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Bin(NativeArrayBuffer::from(" + arg_name.as_str() + ".into_boxed_slice())));\n").as_bytes());
                            },
                            "Vec<u8>" => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Bin(NativeArrayBuffer::from(" + arg_name.as_str() + ".into_boxed_slice())));\n").as_bytes());
                            },
                            "Vec<bool>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for b in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Bool(b));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<i8>" | "Vec<i16>"  => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for n in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Int(n as i32));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<i32>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for n in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Int(n));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<u8>" | "Vec<u16>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for n in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Uint(n as u32));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<u32>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for n in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Uint(n));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<f32>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for n in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Float(n as f64));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<f64>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for n in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Float(n));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<i64>" | "Vec<u64>" | "Vec<i128>" | "Vec<u128>" | "Vec<isize>" | "Vec<usize>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for n in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::BigInt(n.to_bigint().unwrap()));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<BigInt>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for n in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::BigInt(n));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<String>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for str in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Str(str));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<Arc<[u8]>>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for bin in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Bin(NativeArrayBuffer::from(bin.to_vec().into_boxed_slice())));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<Box<[u8]>>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for bin in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Bin(NativeArrayBuffer::from(bin)));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<Arc<Vec<u8>>>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for bin in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Bin(NativeArrayBuffer::from(bin.to_vec().into_boxed_slice())));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<Box<Vec<u8>>>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for bin in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Bin(NativeArrayBuffer::from(bin.into_boxed_slice())));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            "Vec<Vec<u8>>" => {
                                source_content.put_slice((create_tab(level + 3) + "let mut vec = Vec::with_capacity(" + arg_name.as_str() + ".len());\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "for bin in " + arg_name.as_str() + " {\n").as_bytes());
                                source_content.put_slice((create_tab(level + 4) + "vec.push(NativeObjectValue::Bin(NativeArrayBuffer::from(bin.into_boxed_slice())));\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "}\n").as_bytes());
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::Array(vec));\n").as_bytes());
                            },
                            other_type => {
                                source_content.put_slice((create_tab(level + 3) + "args.push(NativeObjectValue::NatObj(NativeObject::new_owned(" + arg_name.as_str() + ")));\n").as_bytes());
                            },
                        }
                        index += 1;
                    }

                    //生成闭包调用后返回结果类型转换为js类型的代码
                    source_content.put_slice((create_tab(level + 3) + "let result: Option<Box<dyn FnOnce(Result<NativeObjectValue, String>) + Send + 'static>> = if let Some(result_callback) = result {\n").as_bytes());
                    source_content.put_slice((create_tab(level + 4) + "Some(Box::new(move |r: Result<NativeObjectValue, String>| {\n").as_bytes());
                    source_content.put_slice((create_tab(level + 5) + "match r {\n").as_bytes());
                    source_content.put_slice((create_tab(level + 6) + "Err(e) => {\n").as_bytes());
                    source_content.put_slice((create_tab(level + 7) + "(result_callback)(Err(e));\n").as_bytes());
                    source_content.put_slice((create_tab(level + 6) + "},\n").as_bytes());
                    source_content.put_slice((create_tab(level + 6) + "Ok(val) => {\n").as_bytes());
                    match result.as_str() {
                        "bool" => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Bool(b) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(b));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        alias@"i8" | alias@"i16" | alias@"i32" | alias@"u8" | alias@"u16" | alias@"u32" | alias@"f32" | alias@"f64"  => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Float(n) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(n as " + alias + "));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        alias@"i64" | alias@"u64" | alias@"i128" | alias@"u128" | alias@"isize" | alias@"usize"  => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::BigInt(n) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(n.to_u128().expect(\"From js bigint to " + alias + " failed\") as " + alias + "));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        "BigInt" => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::BigInt(n) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(n));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        "str" => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Str(str) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(str.as_str()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        "String" => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Str(str) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(str));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        "[u8]" => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Bin(bin) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(bin.bytes()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        "Vec<u8>" => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Bin(bin) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(bin.bytes().to_vec()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        "Vec<bool>" => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Array(array) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "let mut vec = Vec::with_capacity(array.len());\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "while let NativeObjectValue::Bool(b) = array {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 9) + "vec.push(b);\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "}\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(vec));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        alias@"Vec<i8>" | alias@"Vec<i16>" | alias@"Vec<i32>" | alias@"Vec<u8>" | alias@"Vec<u16>" | alias@"Vec<u32>" | alias@"Vec<f32>" | alias@"Vec<f64>" => {
                            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
                            let sub_type = sub_types[1];

                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Array(array) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "let mut vec = Vec::with_capacity(array.len());\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "while let NativeObjectValue::Float(n) = array {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 9) + "vec.push(n as " + sub_type + ");\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "}\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(vec));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        alias@"Vec<i64>" | alias@"Vec<u64>" | alias@"Vec<i128>" | alias@"Vec<u128>" | alias@"Vec<isize>" | alias@"Vec<usize>" => {
                            let sub_types: Vec<&str> = alias.split(|c| c == '<' || c == '>').collect();
                            let sub_type = sub_types[1];

                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Array(array) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "let mut vec = Vec::with_capacity(array.len());\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "while let NativeObjectValue::BigInt(n) = array {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 9) + "vec.push(n.to_u128().expect(\"From js bigint to " + sub_type + " failed\") as " + sub_type + ");\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "}\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(vec));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        "Vec<BigInt>" => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Array(array) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "let mut vec = Vec::with_capacity(array.len());\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "while let NativeObjectValue::BigInt(n) = array {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 9) + "vec.push(n);\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "}\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(vec));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        "Vec<String>" => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Array(array) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "let mut vec = Vec::with_capacity(array.len());\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "while let NativeObjectValue::Str(str) = array {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 9) + "vec.push(str);\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "}\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(vec));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        "Vec<Vec<u8>>" => {
                            source_content.put_slice((create_tab(level + 7) + "if let NativeObjectValue::Array(array) = val {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "let mut vec = Vec::with_capacity(array.len());\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "while let NativeObjectValue::Bin(bin) = array {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 9) + "vec.push(bin.bytes().to_vec());\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "}\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Ok(vec));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 8) + "(result_callback)(Err(\"Parse callback result failed with js function\".to_string()));\n").as_bytes());
                            source_content.put_slice((create_tab(level + 7) + "}\n").as_bytes());
                        },
                        other_type => {
                            return Err(Error::new(ErrorKind::Other, format!("Parse callback result failed with js function, type: {:?}, reason: not support this type", other_type)));
                        },
                    }
                    source_content.put_slice((create_tab(level + 6) + "},\n").as_bytes());
                    source_content.put_slice((create_tab(level + 5) + "}\n").as_bytes());
                    source_content.put_slice((create_tab(level + 4) + "}))\n").as_bytes());
                    source_content.put_slice((create_tab(level + 3) + "} else {\n").as_bytes());
                    source_content.put_slice((create_tab(level + 4) + "None\n").as_bytes());
                    source_content.put_slice((create_tab(level + 3) + "};\n\n").as_bytes());

                    source_content.put_slice((create_tab(level + 3) + "callback.call(args, result);\n").as_bytes());

                    source_content.put_slice((create_tab(level + 2) + "}\n").as_bytes());
                },
            }
            source_content.put_slice((create_tab(level + 1) + ")\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
        other_type => {
            //生成匹配其它类型的只读引用的代码，例: &NativeObject
            source_content.put_slice((create_tab(level) + "NativeObjectValue::NatObj(val) => {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "val\n").as_bytes());
            source_content.put_slice((create_tab(level) + "},\n").as_bytes());
        },
    }

    Ok(())
}

//生成闭包调用参数的代码，返回闭包参数名列表
fn generate_closure_call_args(level: isize,
                              source_content: &mut Vec<u8>,
                              args_type: &Vec<String>,
                              result_type: &String) -> Result<Vec<String>> {
    source_content.put_slice((create_tab(level) + "move |").as_bytes());

    //生成闭包形参列表
    let len = args_type.len() - 1;
    let mut index = 0;
    let mut args_name = Vec::with_capacity(args_type.len());
    for arg_type in args_type {
        let arg_name = (DEFAULT_ARGUMENT_NAME_PREFIX.to_string() + index.to_string().as_str()).to_string();
        match arg_type.as_str() {
            alias@"str" | alias@"[u8]" => {
                //str和[u8]的只读引用作为闭包参数
                source_content.put_slice((arg_name.clone() + ": &" + alias).as_bytes());
            },
            other_type => {
                //其它所有权类型作为闭包参数
                source_content.put_slice((arg_name.clone() + ": " + other_type).as_bytes());
            },
        }

        source_content.put_slice(", ".as_bytes());
        index += 1;

        args_name.push(arg_name);
    }

    //生成闭包调用后返回结果的回调参数
    source_content.put_slice(("result: Option<Box<dyn FnOnce(Result<".to_string() + result_type.as_str() + ", String>) + Send + 'static>>").as_bytes());

    source_content.put_slice("| {\n".as_bytes());

    Ok(args_name)
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
        "()" | "!" => {
            //生成匹配无返回值类型的代码
            if function.is_async() {
                //生成异步返回代码
                source_content.put_slice((create_tab(level) + "reply(Ok(NativeObjectValue::None));\n").as_bytes());
            } else {
                //生成同步返回代码
                source_content.put_slice((create_tab(level) + "return Some(Ok(NativeObjectValue::None));\n").as_bytes());
            }
        },
        "bool" => {
            //生成匹配布尔值类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
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
        alias@"i8" | alias@"i16" | alias@"i32" => {
            //生成匹配有符号整数类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
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
        alias@"i64"=> {
            //生成匹配有符号64位整数类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_i64(r).expect(\"From i64 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_i64(*r).expect(\"From i64 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_i64(*r).expect(\"From i64 to js bigint failed\"))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_i64(r).expect(\"From i64 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_i64(*r).expect(\"From i64 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_i64(*r).expect(\"From i64 to js bigint failed\"))));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"i128" => {
            //生成匹配有符号128位整数类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_i128(r).expect(\"From i128 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_i128(*r).expect(\"From i128 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_i128(*r).expect(\"From i128 to js bigint failed\"))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_i128(r).expect(\"From i128 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_i128(*r).expect(\"From i128 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_i128(*r).expect(\"From i128 to js bigint failed\"))));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"isize" => {
            //生成匹配有符号128位整数类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_isize(r).expect(\"From isize to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_isize(*r).expect(\"From isize to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_isize(*r).expect(\"From isize to js bigint failed\"))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_isize(r).expect(\"From isize to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_isize(*r).expect(\"From isize to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_isize(*r).expect(\"From isize to js bigint failed\"))));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"u8" | alias@"u16" | alias@"u32" => {
            //生成匹配无符号整数类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
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
        alias@"u64" => {
            //生成匹配无符号64位整数类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_u64(r).expect(\"From u64 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_u64(*r).expect(\"From u64 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_u64(*r).expect(\"From u64 to js bigint failed\"))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_u64(r).expect(\"From u64 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_u64(*r).expect(\"From u64 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_u64(*r).expect(\"From u64 to js bigint failed\"))));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"u128" => {
            //生成匹配无符号128位整数类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_u128(r).expect(\"From u128 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_u128(*r).expect(\"From u128 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_u128(*r).expect(\"From u128 to js bigint failed\"))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_u128(r).expect(\"From u128 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_u128(*r).expect(\"From u128 to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_u128(*r).expect(\"From u128 to js bigint failed\"))));\n").as_bytes());
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"usize" => {
            //生成匹配无符号128位整数类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_usize(r).expect(\"From usize to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_usize(*r).expect(\"From usize to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(BigInt::from_usize(*r).expect(\"From usize to js bigint failed\"))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_usize(r).expect(\"From usize to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_usize(*r).expect(\"From usize to js bigint failed\"))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(BigInt::from_usize(*r).expect(\"From usize to js bigint failed\"))));\n").as_bytes());
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
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
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
        alias@"BigInt" => {
            //生成匹配有符号大整数类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(r)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(r.clone())));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::BigInt(r.clone())));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(r)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(r.clone())));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::BigInt(r.clone())));\n").as_bytes());
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
            if let Some(_generic_type) = generic_type {
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
            if let Some(_generic_type) = generic_type {
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
            if let Some(_generic_type) = generic_type {
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
            if let Some(_generic_type) = generic_type {
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
            if let Some(_generic_type) = generic_type {
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
            if let Some(_generic_type) = generic_type {
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
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(level) + "r if r.is::<Box<Vec<u8>>>() => {\n").as_bytes());
                current_level += 1;
            }

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from((*r).into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_only_read() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                } else if return_type.is_writable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from(r.to_vec().into_boxed_slice()))));\n").as_bytes());
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Bin(NativeArrayBuffer::from((*r).into_boxed_slice()))));\n").as_bytes());
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
            if let Some(_generic_type) = generic_type {
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
        "Vec<bool>" => {
            //生成匹配布尔值数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<bool>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::Bool(val));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<bool> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<bool> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<bool> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<bool> type", func_name)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"Vec<i8>" | alias@"Vec<i16>" | alias@"Vec<i32>" => {
            //生成匹配有符号整数数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::Int(val));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"Vec<i64>" => {
            //生成匹配有符号64位整数数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::BigInt(BigInt::from_i64(val).expect(\"From " + alias + " to js bigint array failed\")));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"Vec<i128>" => {
            //生成匹配有符号128位整数数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::BigInt(BigInt::from_i128(val).expect(\"From " + alias + " to js bigint array failed\")));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"Vec<isize>" => {
            //生成匹配有符号128位整数数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::BigInt(BigInt::from_isize(val).expect(\"From " + alias + " to js bigint array failed\")));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"Vec<u16>" | alias@"Vec<u32>" => {
            //生成匹配无符号整数数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::Uint(val));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"Vec<u64>" => {
            //生成匹配无符号64位整数数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::BigInt(BigInt::from_u64(val).expect(\"From " + alias + " to js bigint array failed\")));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"Vec<u128>" => {
            //生成匹配无符号128位整数数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::BigInt(BigInt::from_u128(val).expect(\"From " + alias + " to js bigint array failed\")));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"Vec<usize>" => {
            //生成匹配无符号128位整数数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::BigInt(BigInt::from_usize(val).expect(\"From " + alias + " to js bigint array failed\")));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"Vec<f32>" | alias@"Vec<f64>" => {
            //生成匹配浮点数数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::Float(val));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        alias@"Vec<BigInt>"=> {
            //生成匹配有符号64位整数数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<" + alias + ">() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::BigInt(val));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<{}> type", func_name, alias)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<{}> type", func_name, alias)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<String>" => {
            //生成匹配字符串数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<String>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::Str(val));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<String> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<String> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<String> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<String> type", func_name)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<Arc<[u8]>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<Arc<[u8]>>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::Bin(NativeArrayBuffer::from(val.to_vec().into_boxed_slice())));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Arc<[u8]>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Arc<[u8]>> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Arc<[u8]>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Arc<[u8]>> type", func_name)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<Box<[u8]>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<Box<[u8]>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::Bin(NativeArrayBuffer::from(val)));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Box<[u8]> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Box<[u8]> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Box<[u8]> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Box<[u8]> type", func_name)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<Arc<Vec<u8>>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<Arc<Vec<u8>>>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::Bin(NativeArrayBuffer::from(val.to_vec().into_boxed_slice())));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Arc<Vec<u8>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Arc<Vec<u8>>> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Arc<Vec<u8>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Arc<Vec<u8>>> type", func_name)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<Box<Vec<u8>>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<Box<Vec<u8>>>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::Bin(NativeArrayBuffer::from((*val).into_boxed_slice())));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Box<Vec<u8>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Box<Vec<u8>>> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Box<Vec<u8>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Box<Vec<u8>>> type", func_name)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<Vec<u8>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<Vec<u8>>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(NativeObjectValue::Bin(NativeArrayBuffer::from(val.into_boxed_slice())));\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<u8>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<u8>> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::Array(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<u8>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<u8>> type", func_name)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<Vec<Arc<[u8]>>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<Vec<Arc<[u8]>>>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "let mut arr = Vec::with_capacity(val.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "for v in val {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 2) + "arr.push(NativeObjectValue::Bin(NativeArrayBuffer::from(v.to_vec().into_boxed_slice())));\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "}\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(arr);\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::TwoArray(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<Arc<[u8]>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<Arc<[u8]>>> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::TwoArray(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<Arc<[u8]>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<Arc<[u8]>>> type", func_name)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<Vec<Box<[u8]>>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<Vec<Box<[u8]>>>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "let mut arr = Vec::with_capacity(val.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "for v in val {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 2) + "arr.push(NativeObjectValue::Bin(NativeArrayBuffer::from(v)));\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "}\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(arr);\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::TwoArray(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<Box<[u8]>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<Box<[u8]>>> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::TwoArray(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<Box<[u8]>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<Box<[u8]>>> type", func_name)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<Vec<Arc<Vec<u8>>>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<Vec<Arc<Vec<u8>>>>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "let mut arr = Vec::with_capacity(val.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "for v in val {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 2) + "arr.push(NativeObjectValue::Bin(NativeArrayBuffer::from(v.to_vec().into_boxed_slice())));\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "}\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(arr);\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::TwoArray(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<Arc<Vec<u8>>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<Arc<Vec<u8>>>> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::TwoArray(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<Arc<Vec<u8>>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<Arc<Vec<u8>>>> type", func_name)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<Vec<Box<Vec<u8>>>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<Vec<Box<Vec<u8>>>>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "let mut arr = Vec::with_capacity(val.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "for v in val {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 2) + "arr.push(NativeObjectValue::Bin(NativeArrayBuffer::from((*v).into_boxed_slice())));\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "}\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(arr);\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::TwoArray(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<Box<Vec<u8>>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<Box<Vec<u8>>>> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::TwoArray(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<Box<Vec<u8>>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<Box<Vec<u8>>>> type", func_name)));
                }
            }

            if generic_type.is_some() {
                //泛型的具体类型，则生成匹配项结束
                source_content.put_slice((create_tab(current_level - 1) + "},\n").as_bytes());
            }
        },
        "Vec<Vec<Vec<u8>>>" => {
            //生成匹配二进制缓冲区数组类型的代码
            let mut current_level = level;
            if let Some(_generic_type) = generic_type {
                //泛型的具体类型，则生成匹配项开始
                source_content.put_slice((create_tab(current_level) + "r if r.is::<Vec<Vec<Vec<u8>>>>() => {\n").as_bytes());
                current_level += 1;
            }

            source_content.put_slice((create_tab(current_level) + "let mut array = Vec::with_capacity(r.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "for val in r {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "let mut arr = Vec::with_capacity(val.len());\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "for v in val {\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 2) + "arr.push(NativeObjectValue::Bin(NativeArrayBuffer::from(v.into_boxed_slice())));\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "}\n").as_bytes());
            source_content.put_slice((create_tab(current_level + 1) + "array.push(arr);\n").as_bytes());
            source_content.put_slice((create_tab(current_level) + "}\n").as_bytes());

            if function.is_async() {
                //生成异步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "reply(Ok(NativeObjectValue::TwoArray(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<Vec<u8>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<Vec<u8>>> type", func_name)));
                }
            } else {
                //生成同步返回代码
                if return_type.is_moveable() {
                    source_content.put_slice((create_tab(current_level) + "return Some(Ok(NativeObjectValue::TwoArray(array)));\n").as_bytes());
                } else if return_type.is_only_read() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take onlyread borrow of Vec<Vec<Vec<u8>>> type", func_name)));
                } else if return_type.is_writable() {
                    return Err(Error::new(ErrorKind::Other, format!("Generate function call result failed, function: {}, reason: not allowed take writable borrow of Vec<Vec<Vec<u8>>> type", func_name)));
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