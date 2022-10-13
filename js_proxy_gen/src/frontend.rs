use std::io::{Error, Result, ErrorKind};

use proc_macro2::{TokenTree, TokenStream};
use syn::{self};

use pi_hash::XHashMap;

use crate::utils::{ParseContext,
                   ImportItem,
                   LibPath,
                   ExportItem,
                   Struct,
                   Enum,
                   Function,
                   Const,
                   ConstValue,
                   Type,
                   AttributeTokensFilter,
                   WithParseSpecificTypeStackFrame,
                   LibPathNext};

/*
* 导出标识符
*/
const EXPORT_ATTR_PATH_IDENT: &str = "pi_js_export";

/*
* 泛型的具体类型定义标识符
*/
const TYPE_DEFINED_IDENT: &str = "type";

/*
* 文档标识符
*/
const DOCUMENT_ATTR_PATH_IDENT: &str = "doc";

/*
* 条件编译标识符
*/
const CFG_ATTR_PATH_IDENT: &str = "cfg";

/*
* 特性标识符
*/
const FEATURE_IDENT: &str = "feature";

/*
* 分析指定的Rust源码
*/
pub fn parse_source(context: &mut ParseContext,
                    source: &str) -> Result<()> {
    match syn::parse_file(source) {
        Err(e) => {
            //分析指定的Rust源码失败
            Err(Error::new(ErrorKind::Other, format!("Parse source failed, file: {:?}, reason: {:?}", context.get_origin(), e)))
        },
        Ok(ast) => {
            if let Err(e) = parse_items(context, &ast.items) {
                return Err(Error::new(ErrorKind::Other, format!("Parse source failed, file: {:?}, reason: {:?}", context.get_origin(), e)));
            }

            Ok(())
        },
    }
}

//分析Items
fn parse_items(context: &mut ParseContext,
               items: &Vec<syn::Item>) -> Result<()> {
    for item in items {
        match item {
            syn::Item::Mod(module) => {
                //模块定义，则继续递归分析模块中的所有条目
                if let Some((_, mod_items)) = &module.content {
                    //模块中有条目
                    if let Err(e) = parse_items(context, mod_items) {
                        //分析模块中的条目失败，则立即返回错误
                        return Err(e);
                    }
                }
            },
            syn::Item::Struct(struct_item) => {
                //结构体定义
                match parse_struct(context, struct_item) {
                    Err(e) => {
                        //解析结构体错误，则立即返回错误
                        return Err(e);
                    },
                    Ok(false) => {
                        //解析结构体成功，但未导出，则忽略
                        continue;
                    },
                    _ => (),
                }

                //分析所有为当前结构体定义的实现，包括Trait实现和实现
                for item in items {
                    match item {
                        syn::Item::Impl(impl_item) => {
                            let name = context.get_last_export().unwrap().get_name().unwrap();
                            if let Err(e) = parse_impl(context, &name, impl_item) {
                                return Err(e);
                            }
                        },
                        _ => {
                            //忽略其它词条
                            continue;
                        },
                    }
                }
            },
            syn::Item::Enum(enum_item) => {
                //枚举定义
                match parse_enum(context, enum_item) {
                    Err(e) => {
                        //解析枚举错误，则立即返回错误
                        return Err(e);
                    },
                    Ok(false) => {
                        //解析枚举成功，但未导出，则忽略
                        continue;
                    },
                    _ => (),
                }

                //分析所有为当前枚举定义的实现，包括Trait实现和实现
                for item in items {
                    match item {
                        syn::Item::Impl(impl_item) => {
                            let name = context.get_last_export().unwrap().get_name().unwrap();
                            if let Err(e) = parse_impl(context, &name, impl_item) {
                                return Err(e);
                            }
                        },
                        _ => {
                            //忽略其它词条
                            continue;
                        },
                    }
                }
            },
            syn::Item::Fn(func_item) => {
                //静态函数定义
                match parse_function(context, func_item) {
                    Err(e) => {
                        //解析静态函数错误，则立即返回错误
                        return Err(e);
                    },
                    Ok(false) => {
                        //解析静态函数成功，但未导出，则忽略
                        continue;
                    },
                    _ => (),
                }
            },
            syn::Item::Const(const_item) => {
                //常量定义
                match parse_const(context, const_item) {
                    Err(e) => {
                        //解析常量错误，则立即返回错误
                        return Err(e);
                    },
                    Ok(false) => {
                        //解析常量成功，但未导出，则忽略
                        continue;
                    },
                    _ => (),
                }
            },
            syn::Item::Use(use_item) => {
                //外部库导入定义
                if let Err(e) = parse_import(context, use_item) {
                    return Err(e);
                }
            },
            _ => {
                //不需要分析的词条，则忽略
                continue;
            }
        }
    }

    Ok(())
}

//分析属性
fn parse_attribute(context: &mut ParseContext, attribute: &syn::Attribute) {
    match attribute.parse_meta() {
        Ok(syn::Meta::Path(path)) => {
            //简单属性
            if let Some(ident) = path.get_ident() {
                //有标识符
                parse_attribute_path_tokens(context, attribute, ident);
            }
        },
        Ok(syn::Meta::List(list)) => {
            //属性列表
            if let Some(ident) = list.path.get_ident() {
                //有标识符
                parse_attribute_path_tokens(context, attribute, ident);
            }
        },
        Ok(syn::Meta::NameValue(kv)) => {
            //键值对属性
            if let Some(ident) = kv.path.get_ident() {
                //有标识符
                parse_attribute_path_tokens(context, attribute, ident);
            }
        },
        _ => {
            for segment in attribute.path.segments.iter() {
                parse_attribute_path_tokens(context, attribute, &segment.ident);
            }
        },
    }
}

//分析属性路径和词条流
fn parse_attribute_path_tokens(context: &mut ParseContext,
                               attribute: &syn::Attribute,
                               ident: &syn::Ident) {
    match ident.to_string().as_str() {
        EXPORT_ATTR_PATH_IDENT => {
            //使用了导出属性，则设置当前正在分析的条目为需要导出
            context.set_is_export(true);

            //分析是否有需要导出的泛型的具体类型定义
            if let Some(export_item) = context.get_last_export_mut() {
                for token in &get_attribute_tokens(attribute.tokens.clone(), AttributeTokensFilter::Group as u8) {
                    //在导出属性中定义了泛型的具体类型，则继续分析
                    if let TokenTree::Group(group) = token {
                        for token in &get_attribute_tokens(group.stream(), AttributeTokensFilter::Ident | AttributeTokensFilter::Group) {
                            match token {
                                TokenTree::Ident(ident) => {
                                    //分析泛型名称
                                    let id = ident.to_string();
                                    if id.as_str() == TYPE_DEFINED_IDENT {
                                        //忽略泛型的具体类型定义标识符
                                        continue;
                                    }

                                    export_item.append_generic(id); //记录泛型名称
                                },
                                TokenTree::Group(group) => {
                                    let mut stack = Vec::new();

                                    for token in &get_attribute_tokens(group.stream(), AttributeTokensFilter::Punct as u8 | AttributeTokensFilter::Ident as u8 | AttributeTokensFilter::Group as u8) {
                                        //分析泛型参数的具体类型名称
                                        parse_specific_type(token, &mut stack);
                                    }

                                    for stack_frame in stack {
                                        if let WithParseSpecificTypeStackFrame::Type(specific_type) = stack_frame {
                                            //记录泛型参数的具体类型名称
                                            export_item.append_generic_type(specific_type.to_string());
                                        }
                                    }
                                },
                                _ => (),
                            }
                        }
                    }
                }
            }
        },
        DOCUMENT_ATTR_PATH_IDENT => {
            //使用了文档属性，则记录文档属性的有效词条
            if let Some(export_item) = context.get_last_export_mut() {
                for token in &get_attribute_tokens(attribute.tokens.clone(), AttributeTokensFilter::Literal as u8) {
                    if let TokenTree::Literal(lit) = token {
                        if lit.to_string().trim().len() < 5 {
                            //忽略无效文档，忽略"\\\r"
                            return;
                        }

                        export_item.append_doc(lit.to_string()); //记录有效文档
                    }
                }
            }
        },
        CFG_ATTR_PATH_IDENT => {
            //使用了条件编译属性，则检查是否有名为pi_js_export的feature
            //此分析主要是为了可以导出宏里的待导出条目
            let mut require_export = 0; //是否满足所有导出的条件
            for token in &get_attribute_tokens(attribute.tokens.clone(), AttributeTokensFilter::Group as u8) {
                if let TokenTree::Group(group) = token {
                    for token in &get_attribute_tokens(group.stream(), AttributeTokensFilter::Ident | AttributeTokensFilter::Literal) {
                        match token {
                            TokenTree::Ident(ident) => {
                                //分析特性名称
                                let id = ident.to_string();
                                if id.as_str() != FEATURE_IDENT {
                                    //忽略非特性的名称
                                    continue;
                                }

                                require_export += 1;
                            },
                            TokenTree::Literal(lit) => {
                                if lit.to_string()
                                    .trim_matches(|c| c == ' ' || c == '\"') != EXPORT_ATTR_PATH_IDENT {
                                    //忽略非导出特性值
                                    continue;
                                }

                                require_export += 1;
                            },
                            _ => (),
                        }
                    }
                }
            }

            if require_export < 2 {
                //当前条目不需要导出，则忽略，并继续分析
                return;
            }

            //使用了导出特性，则设置当前正在分析的条目为需要导出
            context.set_is_export(true);
        },
        _ => {

        },
    }
}

//获取属性词条流中的词条
fn get_attribute_tokens(tokens: TokenStream,
                        filter: u8) -> Vec<TokenTree> {
    let mut token_list = Vec::new();

    for token in tokens {
        match token {
            TokenTree::Punct(punct) => {
                //标点符号
                if AttributeTokensFilter::is_no(filter) || AttributeTokensFilter::is_punct(filter) {
                    token_list.push(TokenTree::Punct(punct));
                }
            },
            TokenTree::Ident(ident) => {
                //标识符
                if AttributeTokensFilter::is_no(filter) || AttributeTokensFilter::is_ident(filter) {
                    token_list.push(TokenTree::Ident(ident));
                }
            },
            TokenTree::Literal(literal) => {
                //字面量
                if AttributeTokensFilter::is_no(filter) || AttributeTokensFilter::is_literal(filter) {
                    token_list.push(TokenTree::Literal(literal));
                }
            },
            TokenTree::Group(group) => {
                //词条数组
                if AttributeTokensFilter::is_no(filter) || AttributeTokensFilter::is_group(filter) {
                    token_list.push(TokenTree::Group(group));
                }
            },
        }
    }

    token_list
}

//分析泛型的具体类型
fn parse_specific_type(token: &TokenTree, stack: &mut Vec<WithParseSpecificTypeStackFrame>) {
    match token {
        TokenTree::Punct(punct) => {
            //标点符号
            match punct.as_char() {
                p@'<' => {
                    //记录'<'标识符号
                    stack.push(WithParseSpecificTypeStackFrame::Punct(p));
                },
                '>' => {
                    //具体类型的所有泛型参数，已经分析完成，则从堆栈中弹出对应的类型参数
                    let mut type_args = Vec::new();
                    while let Some(stack_frame) = stack.pop() {
                        match stack_frame {
                            WithParseSpecificTypeStackFrame::Punct('<') => break,
                            WithParseSpecificTypeStackFrame::Type(type_arg) => {
                                type_args.push(type_arg);
                            },
                            _ => continue,
                        }
                    }

                    if let Some(WithParseSpecificTypeStackFrame::Type(specific_type)) = stack.last_mut() {
                        //将类型参数追加到具体类型中
                        while let Some(type_arg) = type_args.pop() {
                            specific_type.append_type_argument(type_arg);
                        }
                    }
                },
                _ => (), //忽略其它标识符号
            }
        },
        TokenTree::Ident(ident) => {
            //标识符
            let r#type = ident.to_string();
            if r#type.trim().len() == 0 {
                //忽略无效类型名称
                return;
            }

            //加入一个具体类型到堆栈
            stack.push(WithParseSpecificTypeStackFrame::Type(Type::new(r#type)));
        },
        TokenTree::Group(group) => {
            //词条数组，仅匹配[T]这种类型，T不允许为有泛型参数的类型
            for token in group.stream() {
                if let TokenTree::Ident(ident) = token {
                    let r#type = ident.to_string();
                    if r#type.trim().len() == 0 {
                        //忽略无效类型名称
                        return;
                    }

                    //加入一个具体类型的分片到堆栈
                    stack.push(WithParseSpecificTypeStackFrame::Type(Type::new("[".to_string() + r#type.as_str() + "]")));
                }
            }
        },
        _ => (),
    }
}

//分析为指定类型名称的Trait实现和实现
fn parse_impl(context: &mut ParseContext,
              name: &String,
              impl_item: &syn::ItemImpl) -> Result<()> {
    let self_ty = &impl_item.self_ty;
    match &**self_ty {
        syn::Type::Path(type_path) => {
            for seg in type_path.path.segments.iter() {
                if &seg.ident.to_string() != name {
                    //如果当前实现的目标类型名称与指定的类型名称不同，则忽略当前实现词条
                    return Ok(());
                }

                break;
            }

            //当前实现的目标类型名称与指定的类型名称相同
            let mut trait_name = None;
            if let Some(trait_) = &impl_item.trait_ {
                //如果是Trait实现，则先分析Trait
                for seq in trait_.1.segments.iter() {
                    trait_name = Some(seq.ident.to_string());
                    break;
                }
            }

            if let Some(trait_name) = &trait_name {
                //有Trait名称，则继续分析导出的Trait方法
                let target_name = if let Some(last_export_item) = context.get_last_export_mut() {
                    last_export_item.append_trait_impl(trait_name.clone());
                    last_export_item.get_type_name().unwrap()
                } else {
                    //不应该执行当前分支
                    unimplemented!();
                };

                if let Err(e) = parse_impl_methods_and_consts(context, &target_name, Some(&trait_name), impl_item) {
                    return Err(e);
                }
            } else {
                //无Trait名称，则继续分析导出的方法
                let target_name = if let Some(last_export_item) = context.get_last_export_mut() {
                    last_export_item.get_type_name().unwrap()
                } else {
                    //不应该执行当前分支
                    unimplemented!();
                };

                if let Err(e) = parse_impl_methods_and_consts(context, &target_name, None, impl_item) {
                    return Err(e);
                }
            }
        },
        _ => {

        },
    }

    Ok(())
}

//分析实现的方法和常量
fn parse_impl_methods_and_consts(context: &mut ParseContext,
                                 target_name: &String,
                                 trait_name: Option<&String>,
                                 impl_item: &syn::ItemImpl) -> Result<()> {
    for item in &impl_item.items {
        match item {
            syn::ImplItem::Method(method_item) => {
                //有方法，则初始化一个导出函数
                context.set_is_export(false); //将当前导出条目的导出设置为未导出
                let f = Function::new();
                let new_export_item = ExportItem::FunctionItem(f);
                context.push_export(new_export_item);

                //遍历方法的所有属性定义，记录文档属性定义和导出定义
                for attr in &method_item.attrs {
                    parse_attribute(context, attr);
                }

                if !context.is_export() {
                    //没有导出定义，则忽略当前方法，并继续分析下一个实现
                    let _ = context.pop_export();
                    continue;
                }

                let mut export_item = context.pop_export().unwrap();
                let method_name = method_item.sig.ident.to_string();
                if trait_name.is_none() {
                    //非Trait方法，需要检查导出方法的可视性
                    if let syn::Visibility::Public(_) = &method_item.vis {
                        //导出的方法为公共可视性，则继续分析
                        ()
                    } else {
                        //无效的导出方法可视性，则立即返回错误
                        return Err(Error::new(ErrorKind::Other, format!("Parse impl method failed, target: {}, trait: {:?}, method: {}, reason: invalid visibility", target_name, trait_name, &method_name)));
                    }
                }

                //有导出定义，则弹出当前导出的函数条目，并继续分析当前方法
                if let ExportItem::FunctionItem(f) = &mut export_item {
                    f.set_name(method_name.clone()); //记录方法名称

                    if let Some(generic) = f.get_generic() {
                        //导出方法的属性中有定义泛型的具体类型，则分析导出方法的泛型声明
                        let mut generic_names = XHashMap::default();
                        for name in generic.get_names() {
                            generic_names.insert(name, ());
                        }

                        for param in &method_item.sig.generics.params {
                            if let syn::GenericParam::Type(tp) = param {
                                let gt = tp.ident.to_string();
                                if let None = generic_names.remove(&gt) {
                                    //导出方法声明的泛型参数在导出属性中没有定义，则立即返回错误
                                    return Err(Error::new(ErrorKind::Other, format!("Parse impl failed, target: {}, trait: {:?}, method: {}, type: {}, reason: undefined generic type in {})", target_name, trait_name, &method_name, &gt, EXPORT_ATTR_PATH_IDENT)));
                                }
                            }
                        }

                        if generic_names.len() > 0 {
                            //导出方法的导出属性中有泛型定义，但在声明中没有对应的泛型参数，则立即返回错误
                            let types: Vec<String> = generic_names.into_iter().map(|(name, _)| {
                                name
                            }).collect();

                            return Err(Error::new(ErrorKind::Other, format!("Parse impl failed, target: {}, trait: {:?}, method: {}, types: {:?}, reason: undeclaration generic type in {}", target_name, trait_name, &method_name, &types, EXPORT_ATTR_PATH_IDENT)));
                        }
                    }

                    //继续分析方法签名
                    if let Err(e) = parse_impl_method_sign(target_name, trait_name, &method_name, f, &method_item.sig) {
                        return Err(e);
                    }
                }

                //将分析完成的导出方法加入，上一个导出对象的实现中
                if let ExportItem::FunctionItem(f) = export_item {
                    if let Some(last_export_item) = context.get_last_export_mut() {
                        if trait_name.is_some() {
                            last_export_item.append_trait_method(f);
                        } else {
                            last_export_item.append_method(f);
                        }
                    }
                }
            },
            syn::ImplItem::Const(const_item) => {
                //有常量，则初始化一个导出常量
                context.set_is_export(false); //将当前导出条目的导出设置为未导出
                let c = Const::new();
                let new_export_item = ExportItem::ConstItem(c);
                context.push_export(new_export_item);

                //遍历常量的所有属性定义，记录文档属性定义和导出定义
                for attr in &const_item.attrs {
                    parse_attribute(context, attr);
                }

                if !context.is_export() {
                    //没有导出定义，则忽略当前常量，并继续分析下一个实现
                    let _ = context.pop_export();
                    continue;
                }

                let mut export_item = context.pop_export().unwrap();
                let const_name = const_item.ident.to_string();
                if let syn::Visibility::Public(_) = &const_item.vis {
                    //导出的常量为公共可视性，则继续分析
                    ()
                } else {
                    //无效的导出常量可视性，则立即返回错误
                    return Err(Error::new(ErrorKind::Other, format!("Parse impl failed, target: {}, trait: {:?}, const: {}, reason: invalid visibility", target_name, trait_name, &const_name)));
                }

                if let ExportItem::ConstItem(c) = &mut export_item {
                    c.set_name(const_name.clone()); //记录常量名称

                    //分析常量的类型
                    match get_type(&"".to_string(), &const_item.ty) {
                        Err(e) => {
                            return Err(e);
                        },
                        Ok(const_type) => {
                            c.set_type(const_type);
                        },
                    }

                    //分析常量的值
                    match get_const_literal(target_name, trait_name, &const_name, &const_item.expr, false) {
                        Err(e) => {
                            return Err(e);
                        },
                        Ok(value) => {
                            c.set_value(value);
                        },
                    }
                }

                //将分析完成的导出常量加入，上一个导出对象的常量列表中
                if let ExportItem::ConstItem(c) = export_item {
                    if let Some(last_export_item) = context.get_last_export_mut() {
                        last_export_item.append_const(c);
                    }
                }
            },
            _ => {
                //忽略实现中的其它词条
                continue;
            },
        }
    }

    Ok(())
}

//分析实现的方法签名
fn parse_impl_method_sign(target_name: &String,
                          trait_name: Option<&String>,
                          method_name: &String,
                          f: &mut Function,
                          method_sign: &syn::Signature) -> Result<()> {
    //分析是否是异步方法
    if let Some(_) = method_sign.asyncness {
        //是异步方法
        f.set_async();
    }

    //分析方法签名的入参
    let args = &method_sign.inputs;
    for arg in args {
        match arg {
            syn::FnArg::Receiver(method_recv) => {
                //接收器参数
                if method_recv.reference.is_some() {
                    //是接收器引用
                    if method_recv.mutability.is_some() {
                        //是可写引用
                        f.append_input("&mut self".to_string(), Type::new(self_to_type(target_name, "Self".to_string())));
                    } else {
                        //是只读引用
                        f.append_input("&self".to_string(), Type::new(self_to_type(target_name, "Self".to_string())));
                    }
                } else {
                    //是接收器所有权
                    if method_recv.mutability.is_some() {
                        //是可写所有权
                        f.append_input("mut self".to_string(), Type::new(self_to_type(target_name, "Self".to_string())));
                    } else {
                        //是只读所有权
                        f.append_input("self".to_string(), Type::new(self_to_type(target_name, "Self".to_string())));
                    }
                }
            },
            syn::FnArg::Typed(arg_typed) => {
                //其它参数
                match &*arg_typed.pat {
                    syn::Pat::Ident(ident) => {
                        //有参数名
                        let arg_name = ident.ident.to_string();
                        match get_input_type(&target_name, trait_name, &method_name, &arg_name, arg_typed) {
                            Err(e) => {
                                //分析参数类型失败，则立即返回错误
                                return Err(e);
                            },
                            Ok(arg_type) => {
                                //分析参数类型成功，则记录方法参数
                                f.append_input(arg_name, arg_type);
                            },
                        }
                    },
                    syn::Pat::Wild(_) => {
                        //只有参数占位符，"_"
                        let arg_name = "_".to_string();
                        match get_input_type(&target_name, trait_name, &method_name, &arg_name, arg_typed) {
                            Err(e) => {
                                //分析参数类型失败，则立即返回错误
                                return Err(e);
                            },
                            Ok(arg_type) => {
                                //分析参数类型成功，则记录方法参数
                                f.append_input(arg_name, arg_type);
                            },
                        }
                    },
                    _ => {
                        //忽略其它词条
                        ()
                    }
                }
            },
        }
    }

    //分析方法签名的出参
    if let syn::ReturnType::Type(_, return_type) = &method_sign.output {
        //有出参
        match get_output_type(target_name, trait_name, &method_name, &*return_type) {
            Err(e) => {
                //分析出参类型失败，则立即返回错误
                return Err(e);
            },
            Ok(ret_type) => {
                //分析出参类型成功，则记录方法出参
                f.set_output(ret_type);
            },
        }
    }

    Ok(())
}

//获取入参类型
fn get_input_type(target_name: &String,
                  trait_name: Option<&String>,
                  method_name: &String,
                  arg_name: &String,
                  arg_typed: &syn::PatType) -> Result<Type> {
    match get_type(target_name, &*arg_typed.ty) {
        Err(e) => {
            //分析入参类型失败，则立即返回错误
            Err(Error::new(ErrorKind::Other, format!("Parse method input type failed, target: {}, trait: {:?}, method: {}, arg: {}, reason: {:?}", target_name, trait_name, method_name, arg_name, e)))
        },
        ok => ok,
    }
}

//获取出参类型
fn get_output_type(target_name: &String,
                   trait_name: Option<&String>,
                   method_name: &String,
                   return_typed: &syn::Type) -> Result<Type> {
    match get_type(target_name, return_typed) {
        Err(e) => {
            //分析出参类型失败，则立即返回错误
            Err(Error::new(ErrorKind::Other, format!("Parse method output type failed, target: {}, trait: {:?}, method: {}, reason: {:?}", target_name, trait_name, method_name, e)))
        },
        ok => ok,
    }
}

//获取类型，将Self类型替换为目标导出条目的类型
fn get_type(target_name: &String,
            r#type: &syn::Type) -> Result<Type> {
    match r#type {
        syn::Type::Path(tp) => {
            //指定类型的所有权
            for seg in &tp.path.segments {
                match &seg.arguments {
                    syn::PathArguments::None => {
                        //类型没有参数
                        return Ok(Type::new(self_to_type(target_name, seg.ident.to_string())));
                    },
                    syn::PathArguments::AngleBracketed(type_args) => {
                        //类型有支持的参数，则分析类型的参数
                        let mut ty = Type::new(self_to_type(target_name, seg.ident.to_string()));

                        let args = &type_args.args;
                        for arg in args {
                            match arg {
                                syn::GenericArgument::Type(type_arg) => {
                                    match get_type(target_name, type_arg) {
                                        Err(e) => {
                                            //分析类型参数失败，则立即返回错误
                                            return Err(e);
                                        },
                                        Ok(t) => {
                                            //分析类型参数成功，则记录
                                            ty.append_type_argument(t);
                                        },
                                    }
                                },
                                _ => (),
                            }
                        }

                        return Ok(ty);
                    },
                    _ => (),
                }
            }
        },
        syn::Type::Slice(ts) => {
            //指定类型的分片
            if let syn::Type::Path(tp) = &*ts.elem {
                for seg in &tp.path.segments {
                    match &seg.arguments {
                        syn::PathArguments::None => {
                            //类型没有参数
                            return Ok(Type::new("[".to_string() + self_to_type(target_name, seg.ident.to_string()).as_str() + "]"));
                        },
                        syn::PathArguments::AngleBracketed(type_args) => {
                            //类型有支持的参数，则分析类型的参数
                            let mut ty = Type::new("[".to_string() + self_to_type(target_name, seg.ident.to_string()).as_str() + "]");

                            let args = &type_args.args;
                            for arg in args {
                                match arg {
                                    syn::GenericArgument::Type(type_arg) => {
                                        match get_type(target_name, type_arg) {
                                            Err(e) => {
                                                //分析类型参数失败，则立即返回错误
                                                return Err(e);
                                            },
                                            Ok(t) => {
                                                //分析类型参数成功，则记录
                                                ty.append_type_argument(t);
                                            },
                                        }
                                    },
                                    _ => (),
                                }
                            }

                            return Ok(ty);
                        },
                        _ => (),
                    }
                }
            }
        },
        syn::Type::Reference(tr) => {
            //指定类型的引用
            let mut type_str = "&".to_string();
            if tr.mutability.is_some() {
                type_str += "mut ";
            }
            match &*tr.elem {
                syn::Type::Path(tp) => {
                    //指定类型的所有权
                    for seg in &tp.path.segments {
                        match &seg.arguments {
                            syn::PathArguments::None => {
                                //类型没有参数
                                type_str += self_to_type(target_name, seg.ident.to_string()).as_str();
                                return Ok(Type::new(type_str));
                            },
                            syn::PathArguments::AngleBracketed(type_args) => {
                                //类型有支持的参数，则分析类型的参数
                                type_str += self_to_type(target_name, seg.ident.to_string()).as_str();
                                let mut ty = Type::new(type_str);

                                let args = &type_args.args;
                                for arg in args {
                                    match arg {
                                        syn::GenericArgument::Type(type_arg) => {
                                            match get_type(target_name, type_arg) {
                                                Err(e) => {
                                                    //分析类型参数失败，则立即返回错误
                                                    return Err(e);
                                                },
                                                Ok(t) => {
                                                    //分析类型参数成功，则记录
                                                    ty.append_type_argument(t);
                                                },
                                            }
                                        },
                                        _ => (),
                                    }
                                }

                                return Ok(ty);
                            },
                            _ => (),
                        }
                    }
                },
                syn::Type::Slice(ts) => {
                    //指定类型的分片
                    if let syn::Type::Path(tp) = &*ts.elem {
                        for seg in &tp.path.segments {
                            match &seg.arguments {
                                syn::PathArguments::None => {
                                    //类型没有参数
                                    type_str = type_str + "[" + self_to_type(target_name, seg.ident.to_string()).as_str() + "]";
                                    return Ok(Type::new(type_str));
                                },
                                syn::PathArguments::AngleBracketed(type_args) => {
                                    //类型有支持的参数，则分析类型的参数
                                    type_str = type_str + "[" + self_to_type(target_name, seg.ident.to_string()).as_str() + "]";
                                    let mut ty = Type::new(type_str);

                                    let args = &type_args.args;
                                    for arg in args {
                                        match arg {
                                            syn::GenericArgument::Type(type_arg) => {
                                                match get_type(target_name, type_arg) {
                                                    Err(e) => {
                                                        //分析类型参数失败，则立即返回错误
                                                        return Err(e);
                                                    },
                                                    Ok(t) => {
                                                        //分析类型参数成功，则记录
                                                        ty.append_type_argument(t);
                                                    },
                                                }
                                            },
                                            _ => (),
                                        }
                                    }

                                    return Ok(ty);
                                },
                                _ => (),
                            }
                        }
                    }
                },
                _ => (),
            }
        },
        syn::Type::TraitObject(to) => {
            for bound in &to.bounds {
                if let syn::TypeParamBound::Trait(tb) = bound {
                    for seg in &tb.path.segments {
                        let ident = seg.ident.to_string();
                        match ident.as_str() {
                            "Fn" | "FnMut" | "FnOnce" => {
                                let mut ty = Type::with(self_to_type(target_name, ident),
                                                        true);

                                match &seg.arguments {
                                    syn::PathArguments::None => {
                                        //函数没有输入参数
                                        return Ok(ty);
                                    },
                                    syn::PathArguments::Parenthesized(type_args) => {
                                        let inputs = &type_args.inputs;
                                        for input in inputs {
                                            match get_type(target_name, input) {
                                                Err(e) => {
                                                    //分析函数输入参数类型失败，则立即返回错误
                                                    return Err(e);
                                                },
                                                Ok(t) => {
                                                    //分析函数输入参数类型成功，则记录
                                                    ty.append_type_argument(t);
                                                },
                                            }
                                        }

                                        return Ok(ty);
                                    },
                                    _ => (),
                                }
                            },
                            _ => (),
                        }
                    }
                }
            }
        },
        syn::Type::Tuple(tt) => {
            //指定类型的元组
            if tt.elems.is_empty() {
                //空元组，即Unit类型
                return Ok(Type::new("()".to_string()));
            }
        },
        syn::Type::Never(_) => {
            //指定!类型
            return Ok(Type::new("!".to_string()));
        },
        _ => (),
    }

    //其它类型，则立即返回错误
    Err(Error::new(ErrorKind::Other, "Parse type failed, reason: invalid type"))
}

//将Self类型替换为目标导出条目的类型
fn self_to_type(target_name: &String, name: String) -> String {
    if name.as_str() == "Self" {
        target_name.clone()
    } else {
        name
    }
}

//获取简单常量字面量表达式的值
pub fn get_const_literal(target_name: &String,
                         trait_name: Option<&String>,
                         const_name: &String,
                         expr: &syn::Expr,
                         neg: bool) -> Result<ConstValue> {
    match expr {
        syn::Expr::Lit(lit) => {
            //字面量表达式
            match &lit.lit {
                syn::Lit::Bool(bool) => {
                    Ok(ConstValue::Boolean(bool.value))
                },
                syn::Lit::Int(int) => {
                    if let Ok(value) = int.base10_parse::<i64>() {
                        if neg {
                            Ok(ConstValue::Int(-value))
                        } else {
                            Ok(ConstValue::Uint(value))
                        }
                    } else {
                        //无效的整数，则立即返回错误
                        Err(Error::new(ErrorKind::Other, format!("Parse const failed, const: {}, reason: invalid const integer", const_name)))
                    }
                },
                syn::Lit::Float(float) => {
                    if let Ok(value) = float.base10_parse::<f64>() {
                        Ok(ConstValue::Float(value))
                    } else {
                        //无效的浮点数，则立即返回错误
                        Err(Error::new(ErrorKind::Other, format!("Parse const failed, const: {}, reason: invalid const float", const_name)))
                    }
                },
                syn::Lit::Str(str) => {
                    Ok(ConstValue::Str(str.value()))
                },
                _ => {
                    //不支持的常量字面值，则立即返回错误
                    Err(Error::new(ErrorKind::Other, format!("Parse const failed, const: {}, reason: invalid const literal", const_name)))
                },
            }
        },
        syn::Expr::Unary(unary) => {
            //一元表达式
            match unary.op {
                syn::UnOp::Neg(_) => {
                    //负号
                    get_const_literal(target_name, trait_name, const_name, &*unary.expr, true)
                },
                _ => {
                    //不支持的一元操作符，则立即返回错误
                    Err(Error::new(ErrorKind::Other, format!("Parse const failed, const: {}, reason: invalid unary op", const_name)))
                },
            }
        },
        _ => {
            //不支持的常量表达式，则立即返回错误
            Err(Error::new(ErrorKind::Other, format!("Parse const failed, const: {}, reason: invalid const expr", const_name)))
        },
    }
}

//分析结构体
fn parse_struct(context: &mut ParseContext,
                struct_item: &syn::ItemStruct) -> Result<bool> {
    //初始化一个导出结构体
    context.set_is_export(false); //将当前导出条目的导出设置为未导出
    let s = Struct::new();
    let export_item = ExportItem::StructItem(s);
    context.push_export(export_item);

    //遍历结构体的所有属性定义，记录文档属性定义和导出定义
    for attr in &struct_item.attrs {
        parse_attribute(context, attr);
    }

    if !context.is_export() {
        //没有导出定义，则弹出当前正在分析的结构体，并立即退出当前结构体的分析
        let _ = context.pop_export();
        return Ok(false);
    }

    let name = struct_item.ident.to_string();

    if let syn::Visibility::Public(_) = &struct_item.vis {
        //导出结构体的为公共可视性，则继续分析
        if let Some(export_item) = context.get_last_export_mut() {
            if let ExportItem::StructItem(s) = export_item {
                s.set_name(name.clone()); //记录结构体名称

                if let Some(generic) = s.get_generic() {
                    //导出结构体的属性中有定义泛型的具体类型，则分析导出结构体的泛型声明
                    let mut generic_names = XHashMap::default();
                    for name in generic.get_names() {
                        generic_names.insert(name, ());
                    }

                    for param in &struct_item.generics.params {
                        if let syn::GenericParam::Type(tp) = param {
                            let gt = tp.ident.to_string();
                            if let None = generic_names.remove(&gt) {
                                //导出结构体声明的泛型参数在导出属性中没有定义，则立即返回错误
                                return Err(Error::new(ErrorKind::Other, format!("Parse struct failed, name: {}, type: {}, reason: undefined generic type in {})", name, gt, EXPORT_ATTR_PATH_IDENT)));
                            }
                        }
                    }

                    if generic_names.len() > 0 {
                        //导出结构体的导出属性中有泛型定义，但在声明中没有对应的泛型参数，则立即返回错误
                        let types: Vec<String> = generic_names.into_iter().map(|(name, _)| {
                            name
                        }).collect();

                        return Err(Error::new(ErrorKind::Other, format!("Parse struct failed, name: {}, types: {:?}, reason: undeclaration generic type in {}", name, types, EXPORT_ATTR_PATH_IDENT)));
                    }
                }
            }
        }

        Ok(true)
    } else {
        //无效的导出结构体可视性，则立即返回错误
        Err(Error::new(ErrorKind::Other, format!("Parse struct failed, name: {}, reason: invalid visibility", name)))
    }
}

//分析枚举
fn parse_enum(context: &mut ParseContext,
              enum_item: &syn::ItemEnum) -> Result<bool> {
    //初始化一个导出枚举
    context.set_is_export(false); //将当前导出条目的导出设置为未导出
    let e = Enum::new();
    let export_item = ExportItem::EnumItem(e);
    context.push_export(export_item);

    //遍历枚举的所有属性定义，记录文档属性定义和导出定义
    for attr in &enum_item.attrs {
        parse_attribute(context, attr);
    }

    if !context.is_export() {
        //没有导出定义，则弹出当前正在分析的枚举，并立即退出当前枚举的分析
        let _ = context.pop_export();
        return Ok(false);
    }

    let name = enum_item.ident.to_string();

    if let syn::Visibility::Public(_) = &enum_item.vis {
        //导出的枚举为公共可视性，则继续分析
        if let Some(export_item) = context.get_last_export_mut() {
            if let ExportItem::EnumItem(e) = export_item {
                e.set_name(name.clone()); //记录枚举名称

                if let Some(generic) = e.get_generic() {
                    //导出枚举的属性中有定义泛型的具体类型，则分析导出枚举的泛型声明
                    let mut generic_names = XHashMap::default();
                    for name in generic.get_names() {
                        generic_names.insert(name, ());
                    }

                    for param in &enum_item.generics.params {
                        if let syn::GenericParam::Type(tp) = param {
                            let gt = tp.ident.to_string();
                            if let None = generic_names.remove(&gt) {
                                //导出枚举声明的泛型参数在导出属性中没有定义，则立即返回错误
                                return Err(Error::new(ErrorKind::Other, format!("Parse enum failed, name: {}, type: {}, reason: undefined generic type in {})", name, gt, EXPORT_ATTR_PATH_IDENT)));
                            }
                        }
                    }

                    if generic_names.len() > 0 {
                        //导出枚举的导出属性中有泛型定义，但在声明中没有对应的泛型参数，则立即返回错误
                        let types: Vec<String> = generic_names.into_iter().map(|(name, _)| {
                            name
                        }).collect();

                        return Err(Error::new(ErrorKind::Other, format!("Parse enum failed, name: {}, types: {:?}, reason: undeclaration generic type in {}", name, types, EXPORT_ATTR_PATH_IDENT)));
                    }
                }
            }
        }

        Ok(true)
    } else {
        //无效的导出枚举可视性，则立即返回错误
        Err(Error::new(ErrorKind::Other, format!("Parse enum failed, name: {}, reason: invalid visibility", name)))
    }
}

//分析静态函数
fn parse_function(context: &mut ParseContext,
                  func_item: &syn::ItemFn) -> Result<bool> {
    //初始化一个导出静态函数
    context.set_is_export(false); //将当前导出条目的导出设置为未导出
    let f = Function::new();
    let export_item = ExportItem::FunctionItem(f);
    context.push_export(export_item);

    //遍历静态函数的所有属性定义，记录文档属性定义和导出定义
    for attr in &func_item.attrs {
        parse_attribute(context, attr);
    }

    if !context.is_export() {
        //没有导出定义，则弹出当前正在分析的静态函数，并立即退出当前静态函数的分析
        let _ = context.pop_export();
        return Ok(false);
    }

    let func_name = func_item.sig.ident.to_string();
    if let syn::Visibility::Public(_) = &func_item.vis {
        //导出的静态函数为公共可视性，则继续分析
        if let Some(ExportItem::FunctionItem(f)) = context.get_last_export_mut() {
            f.set_name(func_name.clone()); //记录静态函数名称

            if let Some(generic) = f.get_generic() {
                //导出静态函数的属性中有定义泛型的具体类型，则分析导出静态函数的泛型声明
                let mut generic_names = XHashMap::default();
                for name in generic.get_names() {
                    generic_names.insert(name, ());
                }

                for param in &func_item.sig.generics.params {
                    if let syn::GenericParam::Type(tp) = param {
                        let gt = tp.ident.to_string();
                        if let None = generic_names.remove(&gt) {
                            //导出静态函数声明的泛型参数在导出属性中没有定义，则立即返回错误
                            return Err(Error::new(ErrorKind::Other, format!("Parse static function failed, function: {}, type: {}, reason: undefined generic type in {})", &func_name, &gt, EXPORT_ATTR_PATH_IDENT)));
                        }
                    }
                }

                if generic_names.len() > 0 {
                    //导出静态函数的导出属性中有泛型定义，但在声明中没有对应的泛型参数，则立即返回错误
                    let types: Vec<String> = generic_names.into_iter().map(|(name, _)| {
                        name
                    }).collect();

                    return Err(Error::new(ErrorKind::Other, format!("Parse static function failed, function: {}, types: {:?}, reason: undeclaration generic type in {}", &func_name, &types, EXPORT_ATTR_PATH_IDENT)));
                }
            }

            //继续分析静态函数签名
            if let Err(e) = parse_impl_method_sign(&"".to_string(), None, &func_name, f, &func_item.sig) {
                return Err(e);
            }
        }

        Ok(true)
    } else {
        //无效的导出静态函数可视性，则立即返回错误
        Err(Error::new(ErrorKind::Other, format!("Parse static function failed, function: {}, reason: invalid visibility", &func_name)))
    }
}

//分析常量
fn parse_const(context: &mut ParseContext,
               const_item: &syn::ItemConst) -> Result<bool> {
    //初始化一个导出常量
    context.set_is_export(false); //将当前导出条目的导出设置为未导出
    let c = Const::new();
    let export_item = ExportItem::ConstItem(c);
    context.push_export(export_item);

    //遍历常量的所有属性定义，记录文档属性定义和导出定义
    for attr in &const_item.attrs {
        parse_attribute(context, attr);
    }

    if !context.is_export() {
        //没有导出定义，则弹出当前正在分析的常量，并立即退出当前常量的分析
        let _ = context.pop_export();
        return Ok(false);
    }

    let const_name = const_item.ident.to_string();
    if let syn::Visibility::Public(_) = &const_item.vis {
        //导出的常量为公共可视性，则继续分析
        if let Some(ExportItem::ConstItem(c)) = context.get_last_export_mut() {
            c.set_name(const_name.clone()); //记录常量名称

            //分析常量的类型
            match get_type(&"".to_string(), &*const_item.ty) {
                Err(e) => {
                    return Err(e);
                },
                Ok(const_type) => {
                    c.set_type(const_type);
                },
            }

            //分析常量的值
            match get_const_literal(&"".to_string(), None, &const_name, &*const_item.expr, false) {
                Err(e) => {
                    return Err(e);
                },
                Ok(value) => {
                    c.set_value(value);
                },
            }
        }

        Ok(true)
    } else {
        //无效的导出常量可视性，则立即返回错误
        Err(Error::new(ErrorKind::Other, format!("Parse const failed, const: {}, reason: invalid visibility", &const_name)))
    }
}

//分析导入声明
pub fn parse_import(context: &mut ParseContext,
                    use_item: &syn::ItemUse) -> Result<()> {
    match &use_item.tree {
        syn::UseTree::Path(path) => {
            let name = path.ident.to_string();
            if let "std" = name.as_str() {
                //导入标准库
                let mut root = LibPath::new(name.clone());
                if let Err(e) = parse_path(&mut root, &*path.tree) {
                    return Err(e);
                }

                let import_item = ImportItem::Std(root);
                context.push_import(import_item);
            } else {
                //导入第三方库
                let mut root = LibPath::new(name.clone());
                if let Err(e) = parse_path(&mut root, &*path.tree) {
                    return Err(e);
                }

                let import_item = ImportItem::Thrid(root);
                context.push_import(import_item);
            }

            Ok(())
        },
        syn::UseTree::Rename(rename) => {
            //导入第三方库的别名
            let mut root = LibPath::new(rename.ident.to_string());
            root.set_alias(rename.rename.to_string());

            let import_item = ImportItem::Thrid(root);
            context.push_import(import_item);

            Ok(())
        },
        _ => {
            //无效的导入声明，则立即返回错误
            Err(Error::new(ErrorKind::Other, "Parse import failed, reason: invalid import"))
        }
    }
}

//分析导入的库路径
pub fn parse_path(prev: &mut LibPath, import: &syn::UseTree) -> Result<()> {
    match import {
        syn::UseTree::Path(path) => {
            //路径
            let mut next = LibPath::new(path.ident.to_string());
            if let Err(e) = parse_path(&mut next, &*path.tree) {
                return Err(e);
            }

            prev.join(LibPathNext::Path(next));
        },
        syn::UseTree::Glob(_) => {
            //所有导出成员
            let end = LibPath::new("*".to_string());
            prev.join(LibPathNext::Path(end));
        },
        syn::UseTree::Group(group) => {
            //组
            let mut vec = Vec::with_capacity(group.items.len());
            for item in &group.items {
                let mut next = LibPath::new("{}".to_string());
                if let Err(e) = parse_path(&mut next, item) {
                    return Err(e);
                }

                vec.push(next);
            }

            prev.join(LibPathNext::Group(vec));
        },
        syn::UseTree::Name(name) => {
            //导出成员
            let end = LibPath::new(name.ident.to_string());
            prev.join(LibPathNext::Path(end));
        },
        syn::UseTree::Rename(rename) => {
            //导出成员别名
            let mut end = LibPath::new(rename.ident.to_string());
            end.set_alias(rename.rename.to_string());
            prev.join(LibPathNext::Path(end));
        },
    }

    Ok(())
}
