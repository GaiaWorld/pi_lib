use std::sync::Arc;
use std::io::{Error, Result, ErrorKind};
use std::path::{Path, PathBuf, Component};

use bytes::BufMut;

#[cfg(feature = "ts_lower_camel_case")]
use heck::AsLowerCamelCase;

use pi_async_file::file::{create_dir, AsyncFile, AsyncFileOptions, WriteOptions};

use crate::{WORKER_RUNTIME,
            utils::{ParseContext, ExportItem, Const, Function, Document, Generic, ConstList, TraitImpls, Impls, TypeName, ProxySourceGenerater, create_tab, get_specific_ts_function_name, get_specific_ts_class_name}};

/*
* 默认的ts本地环境文件名
*/
const DEFAULT_NATIVE_ENV_FILE_NAME: &str = "native_env.d.ts";

/*
* 默认的ts本地环境文件内容
*/
const DEFAULT_NATIVE_ENV_FILE_CONTENT: &str = r#"//本地对象
declare var NativeObject: NativeObjectClass;

//本地对象同步返回值类型
type NativeObjectRetType = undefined|boolean|number|string|ArrayBuffer|ArrayBufferView|Error|object;
//本地对象异步返回值类型
type AsyncNativeObjectRetType = Promise<undefined|boolean|number|string|ArrayBuffer|ArrayBufferView|object>;

declare class NativeObjectClass {
    registry: NativeObjectRegistry; //本地对象回收器注册器
    static_call(index: number, ...anyArgs: any[]): NativeObjectRetType; //本地对象静态同步调用
    async_static_call(index: number, ...anyArgs: any[]): AsyncNativeObjectRetType; //本地对象静态异步调用
    call(index: number, self: object, ...anyArgs: any[]): NativeObjectRetType; //本地对象同步调用
    async_call(index: number, self: object, ...anyArgs: any[]): AsyncNativeObjectRetType; //本地对象异步调用
    release(cid: number, self: object): void; //释放指定的本地对象
}

declare class NativeObjectRegistry {
    //注册指定本地对象的回收器
    register(obj: object, args: [object]): void;
}"#;

/*
* 默认代理ts文件导入的类型
*/
const DEFAULT_PROXY_TS_FILE_USED: &[u8] = b"";

/*
* 在指定的ts文件根目录中创建本地环境文件
*/
pub(crate) async fn generate_public_exports(generate_ts_path: &Path) -> Result<()> {
    match AsyncFile::open(WORKER_RUNTIME.clone(), generate_ts_path.join(DEFAULT_NATIVE_ENV_FILE_NAME), AsyncFileOptions::TruncateWrite).await {
        Err(e) => {
            //创建本地环境文件失败，则立即返回错误
            Err(Error::new(ErrorKind::Other, format!("Generate native env file failed, file: {:?}, reason: {:?}", generate_ts_path, e)))
        },
        Ok(file) => {
            let buf: Arc<[u8]> = Arc::from(DEFAULT_NATIVE_ENV_FILE_CONTENT.as_bytes());
            if let Err(e) = file.write(0, buf, WriteOptions::SyncAll(true)).await {
                //写入本地环境文件内容失败，则立即返回错误
                return Err(Error::new(ErrorKind::Other, format!("Generate native env file failed, file: {:?}, reason: {:?}", generate_ts_path, e)));
            }

            Ok(())
        },
    }
}

/*
* 在指定路径下创建代理的ts文件，并返回异步文件句柄
*/
pub(crate) async fn create_proxy_ts_file(crate_name: String,
                                         source: &ParseContext,
                                         generate_ts_path: &Path) -> Option<Result<(PathBuf, AsyncFile<()>)>> {
    //生成文件路径名
    if source.get_exports().len() == 0 {
        //未导出任何的导出条目，则忽略，并立即退出
        return None;
    }

    let source_path = source.get_origin();
    let mut components = source_path.components();

    let mut b = false;
    let dir_path = generate_ts_path.join(crate_name);
    let mut file_path = PathBuf::new();
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
                file_path = file_path.join(str);
            },
            _ => continue,
        }
    }

    if let Err(e) = create_dir(WORKER_RUNTIME.clone(), dir_path.clone()).await {
        //创建代理js文件所在目录失败，则立即返回错误
        return Some(Err(Error::new(ErrorKind::Other, format!("Create proxy ts file failed, path: {:?}, reason: {:?}", dir_path, e))));
    }

    let filename = if cfg!(windows) {
        file_path.to_str().unwrap().replace(r#"\"#, "_")
    } else {
        file_path.to_str().unwrap().replace(r#"/"#, "_")
    };

    //创建文件
    file_path = dir_path.join(filename);
    let file_path_copy = file_path.clone();
    file_path.set_extension("ts");
    match AsyncFile::open(WORKER_RUNTIME.clone(), file_path, AsyncFileOptions::TruncateWrite).await {
        Err(e) => {
            Some(Err(e))
        },
        Ok(file) => {
            Some(Ok((file_path_copy, file)))
        }
    }
}

//生成ts文件的导入
pub(crate) fn generate_ts_import(mut _path_buf: PathBuf) -> Vec<u8> {
    let source_content = Vec::from(DEFAULT_PROXY_TS_FILE_USED);
    source_content
}

//生成ts文件的所有代理类、代理函数和代理常量的实现
pub(crate) async fn generate_ts_impls(generater: &ProxySourceGenerater,
                                      source: &ParseContext,
                                      source_content: &mut Vec<u8>) -> Result<()> {
    let mut const_items = Vec::new();
    let mut function_items = Vec::new();
    let mut class_items = Vec::new();

    for export_item in source.get_exports() {
        match export_item {
            ExportItem::ConstItem(const_item) => {
                const_items.push(const_item);
            },
            ExportItem::FunctionItem(func_item) => {
                function_items.push(func_item);
            },
            item@ExportItem::StructItem(_) | item@ExportItem::EnumItem(_) => {
                class_items.push(item);
            }
        }
    }

    if let Err(e) = generate_ts_consts(generater, source, const_items, source_content) {
        return Err(e);
    }

    if let Err(e) = generate_ts_functions(generater, source, function_items, source_content).await {
        return Err(e);
    }

    if let Err(e) = generate_ts_classes(generater, source, class_items, source_content).await {
        return Err(e);
    }

    Ok(())
}

//生成ts文件的外部模块常量
fn generate_ts_consts(_generater: &ProxySourceGenerater,
                      _source: &ParseContext,
                      const_items: Vec<&Const>,
                      source_content: &mut Vec<u8>) -> Result<()> {
    for const_item in const_items {
        if let Some(docs) = const_item.get_doc() {
            //生成常量文档
            source_content.put_slice(b"/**\n");
            for doc in docs.get_ref() {
                source_content.put_slice((" *".to_string() + doc.replace("\"", "").as_str() + "\n").as_bytes());
            }
            source_content.put_slice(" */\n".as_bytes());
        }

        if let Some(const_name) = const_item.get_name() {
            //生成常量名称
            source_content.put_slice(("export const ".to_string() + const_name + ": ").as_bytes());

            if let Some(_const_type) = const_item.get_type() {
                if let Some(const_value) = const_item.get_value() {
                    //生成常量类型
                    source_content.put_slice((const_value.get_ts_type_name() + " = ").as_bytes());
                    //生成常量值
                    source_content.put_slice((const_value.to_string().replace(r#"\"#, r#"\\"#) + ";\n\n").as_bytes());
                }
            }
        }
    }

    Ok(())
}

//生成ts文件的外部模块函数
async fn generate_ts_functions(generater: &ProxySourceGenerater,
                               source: &ParseContext,
                               function_items: Vec<&Function>,
                               source_content: &mut Vec<u8>) -> Result<()> {
    for function_item in function_items {
        let functions = if let Some(specific_functions) = function_item.get_specific_functions(){
            //函数有泛型参数，所以获取不同泛型参数的具体类型的所对应的具体函数列表
            specific_functions
        } else {
            vec!(function_item.clone())
        };

        //为当前函数的所有具体函数生成ts代码
        for function in &functions {
            if let Some(docs) = function.get_doc() {
                //生成具体函数文档
                source_content.put_slice(b"/**\n");
                for doc in docs.get_ref() {
                    source_content.put_slice((" *".to_string() + doc.replace("\"", "").as_str() + "\n").as_bytes());
                }
                source_content.put_slice(" */\n".as_bytes());
            }

            //生成具体函数名称
            #[cfg(not(feature = "ts_lower_camel_case"))]
            let function_name = get_specific_ts_function_name(function);
            #[cfg(feature = "ts_lower_camel_case")]
            let function_name = format!("{}", AsLowerCamelCase(get_specific_ts_function_name(function)));

            if function.is_async() {
                //异步函数
                source_content.put_slice(("export async function ".to_string() + function_name.as_str() + "(").as_bytes());
            } else {
                //同步函数
                source_content.put_slice(("export function ".to_string() + function_name.as_str() + "(").as_bytes());
            }

            //生成具体函数的具体入参
            let specific_arg_names = match generate_ts_function_args(None, source, function, source_content) {
                Err(e) => {
                    return Err(e);
                },
                Ok(arg_names) => arg_names,
            };
            source_content.put_slice(b")");

            //生成具体函数的具体出参
            let specific_return_type_name = match generate_ts_function_return(None, None, source, function, source_content) {
                Err(e) => {
                    return Err(e);
                },
                Ok(return_type_name) => return_type_name,
            };

            //生成具体函数体
            let level = 1; //默认的生成函数体的初始层级
            if let Err(e) = generate_specific_function_body(generater, None, None, source, function, function_name, specific_arg_names, specific_return_type_name, level, source_content).await {
                return Err(e);
            }

            source_content.put_slice((create_tab(level - 1) + "}\n\n").as_bytes());
        }
    }

    Ok(())
}

//生成ts文件的所有类
async fn generate_ts_classes(generater: &ProxySourceGenerater,
                             source: &ParseContext,
                             class_items: Vec<&ExportItem>,
                             source_content: &mut Vec<u8>) -> Result<()> {
    for class_item in class_items {
        match class_item {
            ExportItem::StructItem(struct_item) => {
                //目标对象是结构体
                let specific_structs = if let Some(specific_structs) = struct_item.get_specific_structs(){
                    //函数有泛型参数，所以获取不同泛型参数的具体类型的所对应的具体函数列表
                    specific_structs
                } else {
                    vec!(struct_item.clone())
                };

                for specific_struct in specific_structs {
                    if let Err(e) = generate_ts_specific_class(generater,
                                                               source,
                                                               specific_struct.get_doc(),
                                                               specific_struct.get_name(),
                                                               specific_struct.get_consts(),
                                                               specific_struct.get_generic(),
                                                               specific_struct.get_trait_impls(),
                                                               specific_struct.get_impls(),
                                                               source_content).await {
                        return Err(e);
                    }
                }
            },
            ExportItem::EnumItem(enum_item) => {
                //目标对象是枚举
                let specific_enums = if let Some(specific_enums) = enum_item.get_specific_enums(){
                    //函数有泛型参数，所以获取不同泛型参数的具体类型的所对应的具体函数列表
                    specific_enums
                } else {
                    vec!(enum_item.clone())
                };

                for specific_enum in specific_enums {
                    if let Err(e) = generate_ts_specific_class(generater,
                                                               source,
                                                               specific_enum.get_doc(),
                                                               specific_enum.get_name(),
                                                               specific_enum.get_consts(),
                                                               specific_enum.get_generic(),
                                                               specific_enum.get_trait_impls(),
                                                               specific_enum.get_impls(),
                                                               source_content).await {
                        return Err(e);
                    }
                }
            },
            _ => {
                //不应该执行此分支
                unimplemented!();
            }
        }
    }

    Ok(())
}

//生成ts文件的具体类
async fn generate_ts_specific_class(generater: &ProxySourceGenerater,
                                    source: &ParseContext,
                                    doc: Option<&Document>,
                                    class_name: Option<&String>,
                                    consts: Option<&ConstList>,
                                    class_generic: Option<&Generic>,
                                    trait_impls: Option<&TraitImpls>,
                                    impls: Option<&Impls>,
                                    source_content: &mut Vec<u8>) -> Result<()> {
    if let Some(doc) = doc {
        //生成具体类的文档
        source_content.put_slice(b"/**\n");
        for doc_string in doc.get_ref() {
            source_content.put_slice((" *".to_string() + doc_string.replace("\"", "").as_str() + "\n").as_bytes());
        }
        source_content.put_slice(" */\n".as_bytes());
    }

    //生成具体类的名称
    let specific_class_name = get_specific_ts_class_name(class_name.unwrap());
    source_content.put_slice(("export class ".to_string() + specific_class_name.as_str() + " {\n").as_bytes());

    let level = 1; //默认的生成具体类实现的代码层数

    if let Some(const_items) = consts {
        //生成具体类的常量
        for const_item in const_items.get_ref() {
            if let Some(docs) = const_item.get_doc() {
                //生成常量文档
                source_content.put_slice((create_tab(level) + "/**\n").as_bytes());
                for doc in docs.get_ref() {
                    source_content.put_slice((create_tab(level) + " *" + doc.replace("\"", "").as_str() + "\n").as_bytes());
                }
                source_content.put_slice((create_tab(level) + " */\n").as_bytes());
            }

            if let Some(const_name) = const_item.get_name() {
                //生成常量名称
                source_content.put_slice((create_tab(level) + "static readonly " + const_name + ": ").as_bytes());

                if let Some(_const_type) = const_item.get_type() {
                    if let Some(const_value) = const_item.get_value() {
                        //生成常量类型
                        source_content.put_slice((const_value.get_ts_type_name() + " = ").as_bytes());
                        //生成常量值
                        source_content.put_slice((const_value.to_string().replace(r#"\"#, r#"\\"#) + ";\n\n").as_bytes());
                    }
                }
            }
        }
    }

    //生成类的私有域
    source_content.put_slice((create_tab(level) + "/**\n").as_bytes());
    source_content.put_slice((create_tab(level) + " * 本地对象\n").as_bytes());
    source_content.put_slice((create_tab(level) + " */\n").as_bytes());
    source_content.put_slice((create_tab(level) + "private self: object;\n\n").as_bytes());

    //生成类的私有构造方法
    source_content.put_slice((create_tab(level) + "/**\n").as_bytes());
    source_content.put_slice((create_tab(level) + " * 类的私有构造方法\n").as_bytes());
    source_content.put_slice((create_tab(level) + " */\n").as_bytes());
    source_content.put_slice((create_tab(level) + "private constructor(self: object) {\n").as_bytes());
    source_content.put_slice((create_tab(level + 1) + "this.self = self;\n").as_bytes());
    source_content.put_slice((create_tab(level + 1) + "if(NativeObject.registry != undefined) {\n").as_bytes());
    source_content.put_slice((create_tab(level + 2) + "NativeObject.registry.register(self, [self]);\n").as_bytes());
    source_content.put_slice((create_tab(level + 1) + "}\n").as_bytes());
    source_content.put_slice((create_tab(level) + "}\n\n").as_bytes());

    //生成类的获取私有本地对象方法
    source_content.put_slice((create_tab(level) + "/**\n").as_bytes());
    source_content.put_slice((create_tab(level) + " * 获取本地对象方法\n").as_bytes());
    source_content.put_slice((create_tab(level) + " */\n").as_bytes());
    #[cfg(not(feature = "ts_lower_camel_case"))]
    source_content.put_slice((create_tab(level) + "public get_self() {\n").as_bytes());
    #[cfg(feature = "ts_lower_camel_case")]
    source_content.put_slice((create_tab(level) + "public getSelf() {\n").as_bytes());
    source_content.put_slice((create_tab(level + 1) + "return this.self;\n").as_bytes());
    source_content.put_slice((create_tab(level) + "}\n\n").as_bytes());

    //生成类的释放方法
    source_content.put_slice((create_tab(level) + "/**\n").as_bytes());
    source_content.put_slice((create_tab(level) + " * 释放本地对象的方法\n").as_bytes());
    source_content.put_slice((create_tab(level) + " */\n").as_bytes());
    source_content.put_slice((create_tab(level) + "public destroy() {\n").as_bytes());
    source_content.put_slice((create_tab(level + 1) + "if(this.self == undefined) {\n").as_bytes());
    source_content.put_slice((create_tab(level + 2) + "throw new Error(\"" + specific_class_name.as_str() + " already destroy\");\n").as_bytes());
    source_content.put_slice((create_tab(level + 1) + "}\n\n").as_bytes());
    source_content.put_slice((create_tab(level + 1) + "NativeObject.release(_$cid, this.self);\n").as_bytes());
    source_content.put_slice((create_tab(level + 1) + "this.self = undefined;\n").as_bytes());
    source_content.put_slice((create_tab(level) + "}\n\n").as_bytes());

    //生成类的从指定本地对象构建当前类方法
    source_content.put_slice((create_tab(level) + "/**\n").as_bytes());
    source_content.put_slice((create_tab(level) + " * 从指定本地对象构建当前类方法，此方法是不安全的，使用错误的本地对象将会导致调用时异常\n").as_bytes());
    source_content.put_slice((create_tab(level) + " */\n").as_bytes());
    source_content.put_slice((create_tab(level) + "static from(obj: object): " + specific_class_name.as_str() + " {\n").as_bytes());
    source_content.put_slice((create_tab(level + 1) + "return new " + specific_class_name.as_str() + "(obj);\n").as_bytes());
    source_content.put_slice((create_tab(level) + "}\n\n").as_bytes());

    if let Some(trait_impls) = trait_impls {
        //生成类的Trait方法
        for (_, functions) in trait_impls.get_ref() {
            if let Err(e) = generate_ts_specific_class_method(generater, source, class_name.unwrap(), class_generic, functions.as_slice(), level, source_content).await {
                return Err(e);
            }
        }
    }

    if let Some(impls) = impls {
        //生成类的方法
        if let Err(e) = generate_ts_specific_class_method(generater, source, class_name.unwrap(), class_generic, impls.get_ref(), level, source_content).await {
            return Err(e);
        }
    }

    source_content.put_slice(b"}\n\n");

    Ok(())
}

//生成ts文件的具体类的具体方法
async fn generate_ts_specific_class_method(generater: &ProxySourceGenerater,
                                           source: &ParseContext,
                                           specific_class_name: &String,
                                           specific_class_generic: Option<&Generic>,
                                           function_items: &[Function],
                                           level: isize,
                                           source_content: &mut Vec<u8>) -> Result<()> {
    for function_item in function_items {
        let functions = if let Some(specific_functions) = function_item.get_specific_functions(){
            //方法有泛型参数，所以获取不同泛型参数的具体类型的所对应的具体方法列表
            specific_functions
        } else {
            vec!(function_item.clone())
        };

        //为当前方法的所有具体方法生成ts代码
        for function in &functions {
            if let Some(docs) = function.get_doc() {
                //生成具体方法文档
                source_content.put_slice((create_tab(level) + "/**\n").as_bytes());
                for doc in docs.get_ref() {
                    source_content.put_slice((create_tab(level) + " *" + doc.replace("\"", "").as_str() + "\n").as_bytes());
                }
                source_content.put_slice((create_tab(level) + " */\n").as_bytes());
            }

            //生成具体方法名称
            #[cfg(not(feature = "ts_lower_camel_case"))]
            let function_name = get_specific_ts_function_name(function);
            #[cfg(feature = "ts_lower_camel_case")]
            let function_name = format!("{}", AsLowerCamelCase(get_specific_ts_function_name(function)));

            if function.is_static() {
                //静态方法
                if function.is_async() {
                    //异步静态方法
                    source_content.put_slice((create_tab(level) + "static async " + function_name.as_str() + "(").as_bytes());
                } else {
                    //同步静态函数
                    source_content.put_slice((create_tab(level) + "static " + function_name.as_str() + "(").as_bytes());
                }
            } else {
                //方法
                if function.is_async() {
                    //异步方法
                    source_content.put_slice((create_tab(level) + "public async " + function_name.as_str() + "(").as_bytes());
                } else {
                    //同步方法
                    source_content.put_slice((create_tab(level) + "public " + function_name.as_str() + "(").as_bytes());
                }
            }

            //生成具体方法的具体入参
            let specific_arg_names = match generate_ts_function_args(specific_class_generic, source, function, source_content) {
                Err(e) => {
                    return Err(e);
                },
                Ok(arg_names) => arg_names,
            };
            source_content.put_slice(b")");

            //生成具体方法的具体出参
            let specific_return_type_name = match generate_ts_function_return(Some(specific_class_name), specific_class_generic, source, function, source_content) {
                Err(e) => {
                    return Err(e);
                },
                Ok(return_type_name) => return_type_name,
            };

            //生成具体方法体
            if let Err(e) = generate_specific_function_body(generater, Some(specific_class_name), specific_class_generic, source, function, function_name, specific_arg_names, specific_return_type_name, level + 1, source_content).await {
                return Err(e);
            }

            source_content.put_slice((create_tab(level) + "}\n\n").as_bytes());
        }
    }

    Ok(())
}

//生成ts文件的函数或方法的入参
fn generate_ts_function_args(generic: Option<&Generic>,
                             _source: &ParseContext,
                             function: &Function,
                             source_content: &mut Vec<u8>) -> Result<Vec<String>> {
    let mut specific_arg_names = Vec::new(); //具体参数名称列表

    if let Some(input) = function.get_input() {
        //函数有参数
        let mut index = 0; //可忽略参数序号
        let args = if function.is_static() {
            //静态函数或静态方法的参数
            input.get_ref()
        } else {
            //方法的参数
            &input.get_ref()[1..]
        };
        let mut args_len = args.len(); //参数数量


        for (arg_name, arg_type) in args {
            let last_args_len = args_len;
            let specific_arg_name = get_specific_arg_name(arg_name, index);
            index += 1;

            if let Some(generic) = generic {
                //目标对象有泛型参数
                for (generic_name, specific_types) in generic.get_ref() {
                    if arg_type.get_type_name().get_name() == generic_name {
                        //泛型参数名相同，则使用具体类型替换泛型类型
                        let specific_arg_type_name = get_ts_type_name(specific_types[0].get_name().as_str());
                        specific_arg_names.push(specific_arg_name.clone());
                        source_content.put_slice((specific_arg_name.clone() + ": " + specific_arg_type_name.as_str()).as_bytes());
                        args_len -= 1; //已生成指定参数，则减少未生成的参数数量
                        break;
                    }
                }

                if last_args_len > args_len {
                    if args_len > 0 {
                        //根据参数数量生成参数分隔符
                        source_content.put_slice(b", ");
                    }

                    //继续下一个参数的处理
                    continue;
                }
            }

            if let Some(generic) = function.get_generic() {
                //函数有泛型参数
                for (generic_name, specific_types) in generic.get_ref() {
                    if arg_type.get_type_name().get_name() == generic_name {
                        //泛型参数名相同，则使用具体类型替换泛型类型
                        let specific_arg_type_name = get_ts_type_name(specific_types[0].get_name().as_str());
                        specific_arg_names.push(specific_arg_name.clone());
                        source_content.put_slice((specific_arg_name.clone() + ": " + specific_arg_type_name.as_str()).as_bytes());
                        args_len -= 1; //已生成指定参数，则减少未生成的参数数量
                        break;
                    }
                }

                if last_args_len > args_len {
                    if args_len > 0 {
                        //根据参数数量生成参数分隔符
                        source_content.put_slice(b", ");
                    }

                    //继续下一个参数的处理
                    continue;
                }
            }

            //没有任何泛型参数
            let specific_arg_type_name = get_ts_type_name(arg_type.get_type_name().get_name().as_str());
            specific_arg_names.push(specific_arg_name.clone());
            source_content.put_slice((specific_arg_name + ": " + specific_arg_type_name.as_str()).as_bytes());
            args_len -= 1; //已生成指定参数，则减少未生成的参数数量

            //根据参数数量生成参数分隔符
            if args_len > 0 {
                source_content.put_slice(b", ");
            }
        }
    }

    Ok(specific_arg_names)
}

//生成ts文件的函数或方法的出参
fn generate_ts_function_return(target: Option<&String>,
                               _generic: Option<&Generic>,
                               _source: &ParseContext,
                               function: &Function,
                               source_content: &mut Vec<u8>) -> Result<Option<String>> {
    if let Some(return_type) = function.get_output() {
        //函数有返回值
        let ( return_type_name, other_return_type_name) = match return_type.get_part_type_name() {
            TypeName::Moveable(part_type_name) => {
                match part_type_name.as_str() {
                    "Option" => {
                        if function.is_async() {
                            //异步函数
                            (return_type.get_type_arg_names().unwrap()[0].clone(), "".to_string())
                        } else {
                            //同步函数
                            (return_type.get_type_arg_names().unwrap()[0].clone(), "|undefined".to_string())
                        }
                    },
                    "Result" => {
                        if function.is_async() {
                            //异步函数
                            (return_type.get_type_arg_names().unwrap()[0].clone(), "".to_string())
                        } else {
                            //同步函数
                            (return_type.get_type_arg_names().unwrap()[0].clone(), "|Error".to_string())
                        }
                    },
                    _ => (return_type.get_type_name(), "".to_string()),
                }
            },
            TypeName::OnlyRead(part_type_name) => {
                match part_type_name.as_str() {
                    "Option" => {
                        return Err(Error::new(ErrorKind::Other, format!("Generate ts function return failed, function: {}, reason: not allowed take owner of Option type", function.get_name().unwrap())));
                    },
                    "Result" => {
                        return Err(Error::new(ErrorKind::Other, format!("Generate ts function return failed, function: {}, reason: not allowed take only read borrow of Option type", function.get_name().unwrap())));
                    },
                    _ => (return_type.get_type_name(), "".to_string()),
                }
            },
            TypeName::Writable(part_type_name) => {
                match part_type_name.as_str() {
                    "Option" => {
                        return Err(Error::new(ErrorKind::Other, format!("Generate ts function return failed, function: {}, reason: not allowed take owner of Option type", function.get_name().unwrap())));
                    },
                    "Result" => {
                        return Err(Error::new(ErrorKind::Other, format!("Generate ts function return failed, function: {}, reason: not allowed take only read borrow of Option type", function.get_name().unwrap())));
                    },
                    _ => (return_type.get_type_name(), "".to_string()),
                }
            },
        };

        let specific_return_type_name = if let Some(target_name) = target {
            //有目标对象
            if filter_type_args(target_name) == filter_type_args(return_type_name.get_name()) {
                //返回类型为目标对象
                if function.is_static() {
                    //静态函数，则返回目标对象的具体类型名
                    get_specific_ts_class_name(target_name)
                } else {
                    //函数，则返回本地对象类型名
                    get_ts_type_name(return_type_name.get_name().as_str())
                }
            } else {
                //返回类型为其它类型
                get_ts_type_name(return_type_name.get_name().as_str())
            }
        } else {
            //没有目标对象，则返回本地对象类型名
            get_ts_type_name(return_type_name.get_name().as_str())
        };

        if function.is_async() {
            //异步函数
            source_content.put_slice((": Promise<".to_string() + specific_return_type_name.as_str() + "> {\n").as_bytes());
        } else {
            //同步函数
            source_content.put_slice((": ".to_string() + specific_return_type_name.as_str() + other_return_type_name.as_str() + " {\n").as_bytes());
        }

        Ok(Some(specific_return_type_name))
    } else {
        //函数没有返回值
        if function.is_async() {
            //异步函数
            source_content.put_slice(b": Promise<undefined> {\n");
            Ok(Some("Promise<undefined>".to_string()))
        } else {
            //同步函数
            source_content.put_slice(b": void {\n");
            Ok(None)
        }
    }
}

//生成函数体
async fn generate_specific_function_body(generater: &ProxySourceGenerater,
                                         target: Option<&String>,
                                         _generic: Option<&Generic>,
                                         _source: &ParseContext,
                                         function: &Function,
                                         specific_function_name: String,
                                         specific_arg_names: Vec<String>,
                                         specific_return_type_name: Option<String>,
                                         level: isize,
                                         source_content: &mut Vec<u8>) -> Result<()> {
    if let Some(target_name) = target {
        //有目标对象
        if function.is_static() {
            if function.is_async() {
                //异步静态方法
                if let Some(method_index) = generater.get_async_static_method_index(target_name.clone(), specific_function_name).await {
                    if specific_return_type_name.is_some() {
                        //异步静态方法有返回值
                        source_content.put_slice((create_tab(level) + "let result = NativeObject.async_static_call(" + method_index.to_string().as_str()).as_bytes());
                    } else {
                        //异步静态方法没有有返回值
                        source_content.put_slice((create_tab(level) + "NativeObject.async_static_call(" + method_index.to_string().as_str()).as_bytes());
                    }

                    for specific_arg_name in specific_arg_names {
                        source_content.put_slice((", ".to_string() + specific_arg_name.as_str()).as_bytes());
                    }

                    if let Some(specific_return_type_name) = &specific_return_type_name {
                        //异步静态方法有返回值
                        if specific_return_type_name == &get_specific_ts_class_name(target_name) {
                            //当前异步静态方法的返回值类型与目标对象的具体类型相同，则当前异步静态方法是当前目标对象的构造方法
                            source_content.put_slice(b") as Promise<object>;\n");
                            source_content.put_slice((create_tab(level) + "let r: object = await result;\n").as_bytes());
                            source_content.put_slice((create_tab(level) + "if(r instanceof Error) {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 1) + "throw r;\n").as_bytes());
                            source_content.put_slice((create_tab(level) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 1) + "return new " + specific_return_type_name + "(r);\n").as_bytes());
                            source_content.put_slice((create_tab(level) + "}\n").as_bytes());
                        } else {
                            //当前异步静态方法，不是当前目标对象的构造方法
                            source_content.put_slice((") as Promise<".to_string() + specific_return_type_name + ">;\n").as_bytes());
                            source_content.put_slice((create_tab(level) + "return await result;\n").as_bytes());
                        }
                    } else {
                        //异步静态函数没有有返回值
                        source_content.put_slice(b");\n")
                    }
                }
            } else {
                //同步静态方法
                if let Some(method_index) = generater.get_static_method_index(target_name.clone(), specific_function_name).await {
                    if specific_return_type_name.is_some() {
                        //同步静态方法有返回值
                        source_content.put_slice((create_tab(level) + "let result = NativeObject.static_call(" + method_index.to_string().as_str()).as_bytes());
                    } else {
                        //同步静态方法没有有返回值
                        source_content.put_slice((create_tab(level) + "NativeObject.static_call(" + method_index.to_string().as_str()).as_bytes());
                    }

                    //生成其它入参
                    for specific_arg_name in specific_arg_names {
                        source_content.put_slice((", ".to_string() + specific_arg_name.as_str()).as_bytes());
                    }

                    if let Some(specific_return_type_name) = &specific_return_type_name {
                        //同步静态方法有返回值
                        if specific_return_type_name == &get_specific_ts_class_name(target_name) {
                            //当前同步静态方法的返回值类型与目标对象的具体类型相同，则当前同步静态方法是当前目标对象的构造方法
                            source_content.put_slice(b") as object;\n");
                            source_content.put_slice((create_tab(level) + "if(result instanceof Error) {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 1) + "throw result;\n").as_bytes());
                            source_content.put_slice((create_tab(level) + "} else {\n").as_bytes());
                            source_content.put_slice((create_tab(level + 1) + "return new " + specific_return_type_name + "(result);\n").as_bytes());
                            source_content.put_slice((create_tab(level) + "}\n").as_bytes());
                        } else {
                            //当前同步静态方法，不是当前目标对象的构造方法
                            source_content.put_slice((") as ".to_string() + specific_return_type_name + ";\n").as_bytes());
                            source_content.put_slice((create_tab(level) + "return result;\n").as_bytes());
                        }
                    } else {
                        //同步静态方法没有有返回值
                        source_content.put_slice(b");\n")
                    }
                }
            }
        } else {
            //方法，则生成检查代码
            source_content.put_slice((create_tab(level) + "if(this.self == undefined) {\n").as_bytes());
            source_content.put_slice((create_tab(level + 1) + "throw new Error(\"" + get_specific_ts_class_name(target_name).as_str() + " object already destroy\");\n").as_bytes());
            source_content.put_slice((create_tab(level) + "}\n\n").as_bytes());

            if function.is_async() {
                //异步方法
                if let Some(method_index) = generater.get_async_method_index(target_name.clone(), specific_function_name).await {
                    if specific_return_type_name.is_some() {
                        //异步方法有返回值
                        source_content.put_slice((create_tab(level) + "let result = NativeObject.async_call(" + method_index.to_string().as_str() + ", this.self").as_bytes());
                    } else {
                        //异步方法没有有返回值
                        source_content.put_slice((create_tab(level) + "NativeObject.async_call(" + method_index.to_string().as_str() + ", this.self").as_bytes());
                    }

                    //生成其它入参
                    for specific_arg_name in specific_arg_names {
                        source_content.put_slice((", ".to_string() + specific_arg_name.as_str()).as_bytes());
                    }

                    if let Some(specific_return_type_name) = &specific_return_type_name {
                        //异步方法有返回值
                        source_content.put_slice((") as Promise<".to_string() + specific_return_type_name + ">;\n").as_bytes());
                        source_content.put_slice((create_tab(level) + "return await result;\n").as_bytes());
                    } else {
                        //异步方法没有有返回值
                        source_content.put_slice(b");\n")
                    }
                }
            } else {
                //同步方法
                if let Some(method_index) = generater.get_method_index(target_name.clone(), specific_function_name).await {
                    if specific_return_type_name.is_some() {
                        //同步方法有返回值
                        source_content.put_slice((create_tab(level) + "let result = NativeObject.call(" + method_index.to_string().as_str() + ", this.self").as_bytes());
                    } else {
                        //同步方法没有有返回值
                        source_content.put_slice((create_tab(level) + "NativeObject.call(" + method_index.to_string().as_str() + ", this.self").as_bytes());
                    }

                    //生成其它入参
                    for specific_arg_name in specific_arg_names {
                        source_content.put_slice((", ".to_string() + specific_arg_name.as_str()).as_bytes());
                    }

                    if let Some(specific_return_type_name) = &specific_return_type_name {
                        //同步方法有返回值
                        source_content.put_slice((") as ".to_string() + specific_return_type_name + ";\n").as_bytes());
                        source_content.put_slice((create_tab(level) + "return result;\n").as_bytes());
                    } else {
                        //同步方法没有有返回值
                        source_content.put_slice(b");\n")
                    }
                }
            }
        }
    } else {
        //没有目标对象
        if function.is_async() {
            //异步静态函数
            if let Some(method_index) = generater.get_async_static_method_index("".to_string(), specific_function_name).await {
                if specific_return_type_name.is_some() {
                    //异步静态函数有返回值
                    source_content.put_slice((create_tab(level) + "let result = NativeObject.async_static_call(" + method_index.to_string().as_str()).as_bytes());
                } else {
                    //异步静态函数没有有返回值
                    source_content.put_slice((create_tab(level) + "NativeObject.async_static_call(" + method_index.to_string().as_str()).as_bytes());
                }

                //生成其它入参
                for specific_arg_name in specific_arg_names {
                    source_content.put_slice((", ".to_string() + specific_arg_name.as_str()).as_bytes());
                }

                if let Some(specific_return_type_name) = &specific_return_type_name {
                    //异步静态函数有返回值
                    source_content.put_slice((") as Promise<".to_string() + specific_return_type_name + ">;\n").as_bytes());
                    source_content.put_slice((create_tab(level) + "return await result;\n").as_bytes());
                } else {
                    //异步静态函数没有有返回值
                    source_content.put_slice(b");\n")
                }
            }
        } else {
            //同步静态函数
            if let Some(method_index) = generater.get_static_method_index("".to_string(), specific_function_name).await {
                if specific_return_type_name.is_some() {
                    //同步静态函数有返回值
                    source_content.put_slice((create_tab(level) + "let result = NativeObject.static_call(" + method_index.to_string().as_str()).as_bytes());
                } else {
                    //同步静态函数没有有返回值
                    source_content.put_slice((create_tab(level) + "NativeObject.static_call(" + method_index.to_string().as_str()).as_bytes());
                }

                //生成其它入参
                for specific_arg_name in specific_arg_names {
                    source_content.put_slice((", ".to_string() + specific_arg_name.as_str()).as_bytes());
                }

                if let Some(specific_return_type_name) = &specific_return_type_name {
                    //同步静态函数有返回值
                    source_content.put_slice((") as ".to_string() + specific_return_type_name + ";\n").as_bytes());
                    source_content.put_slice((create_tab(level) + "return result;\n").as_bytes());
                } else {
                    //同步静态函数没有有返回值
                    source_content.put_slice(b");\n")
                }
            }
        }
    }

    Ok(())
}

//获取具体参数名，如果参数名为"_"，则在后面追加一个数字，以表示可以忽略的参数名
fn get_specific_arg_name(arg_name: &String, index: usize) -> String {
    if arg_name == "_" {
        arg_name.clone() + index.to_string().as_str()
    } else {
        arg_name.clone()
    }
}

//获取具体类型的ts类型名
fn get_ts_type_name(specific_arg_type_name: &str) -> String {
    match specific_arg_type_name {
        "bool" => "boolean".to_string(),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "f32" | "f64" => "number".to_string(),
        "str" | "String" => "string".to_string(),
        "[u8]" | "Arc<[u8]>" | "Box<[u8]>" | "Arc<Vec<u8>>" | "Box<Vec<u8>>" | "Vec<u8>" => "ArrayBuffer".to_string(),
        "Vec<bool>" => "boolean[]".to_string(),
        "Vec<i8>" | "Vec<i16>" | "Vec<i32>" | "Vec<i64>" | "Vec<i128>" | "Vec<isize>" | "Vec<u16>" | "Vec<u32>" | "Vec<u64>" | "Vec<u128>" | "Vec<usize>" | "Vec<f32>" | "Vec<f64>" => "number[]".to_string(),
        "Vec<String>" => "string[]".to_string(),
        "Vec<Arc<[u8]>>" | "Vec<Box<[u8]>>" | "Vec<Arc<Vec<u8>>>" | "Vec<Box<Vec<u8>>>" | "Vec<Vec<u8>>" => "ArrayBuffer[]".to_string(),
        "Vec<Vec<Arc<[u8]>>>" | "Vec<Vec<Box<[u8]>>>" | "Vec<Vec<Arc<Vec<u8>>>>" | "Vec<Vec<Box<Vec<u8>>>>" | "Vec<Vec<Vec<u8>>>" => "ArrayBuffer[][]".to_string(),
        _ => "object".to_string(),
    }
}

//过滤指定类型的所有类型参数
fn filter_type_args(type_name: &str) -> String {
    let vec: Vec<&str> = type_name.split('<').collect();
    vec[0].to_string()
}