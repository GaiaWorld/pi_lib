//! # 定义了用于标注需要导出到js虚拟机的Rust代码的过程宏
//!
use std::collections::{VecDeque, BTreeMap};

use proc_macro::TokenStream;
use std::any::Any;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree, Delimiter};
use quote::{quote, quote_spanned, ToTokens};
use syn::{DeriveInput, ItemEnum, Visibility, Ident,
          parse::{Parse, ParseStream, Parser}};
use syn::spanned::Spanned;

// 属性参数列表
type AttributeArgs = syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>;

///
/// js代码生成器使用的导出属性
///
#[proc_macro_attribute]
pub fn pi_js_export(attrs: TokenStream,
                    items: TokenStream) -> TokenStream {
    match syn::parse2::<ItemEnum>(items.clone().into()) {
        Err(_) => {
            //不是枚举，则立即原样返回
            items
        },
        Ok(input) => {
            //是枚举
            if let Visibility::Public(_) = &input.vis {
                //是公开的枚举
                match AttributeArgs::parse_terminated
                    .parse2(attrs.into()) {
                    Err(_) => items,
                    Ok(args) => {
                        parse_c_like_enum_macro(input.clone(), args).into()
                    }
                }
            } else {
                //不是公开的枚举，则立即原样返回
                items
            }
        },
    }
}

// 分析类C枚举
fn parse_c_like_enum_macro(item: ItemEnum,
                           args: AttributeArgs) -> TokenStream2 {
    let mut stack = Vec::with_capacity(2);
    let mut buf: VecDeque<(String, i32)> = VecDeque::new();

    let mut is_c_like_enum = true;
    let mut variants_punct_len = 0;
    let variants_len = item.variants.len();
    for token in item.variants.to_token_stream() {
        match token {
            TokenTree::Ident(ident) => {
                stack.push(ident.to_string());
            },
            TokenTree::Literal(lit) => {
                stack.push(lit.to_string());
            },
            TokenTree::Punct(punct) => {
                if punct.as_char() != ',' {
                    continue;
                }
                variants_punct_len += 1;

                let (key, value) = match stack.len() {
                    1 => {
                        if let Some((_key, last_value)) = buf.back() {
                            //无字面值，则获取上一个枚举成员的值加1，并设置为当前成员的值
                            match (*last_value).checked_add(1) {
                                None => {
                                    //越界，则立即返回错误原因
                                    return token_stream_with_error(item.to_token_stream(),
                                                                   syn::Error::new_spanned(&item.ident,
                                                                                           format!("Parse c-like enum failed, last_value: {:?}, reason: integer overflow",
                                                                                                   last_value)));
                                },
                                Some(current_value) => {
                                    (stack.pop().unwrap(), current_value)
                                },
                            }
                        } else {
                            //无字面值，且没有上一个枚举成员的值
                            (stack.pop().unwrap(), 0)
                        }
                    },
                    2 => {
                        //有字面值，则赋值
                        let val = stack
                            .pop()
                            .unwrap()
                            .replace("_", "");
                        let value = match val.parse::<i32>() {
                            Err(_) => {
                                match val.strip_prefix("0b") {
                                    None => {
                                        match val.strip_prefix("0o") {
                                            None => {
                                                match val.strip_prefix("0x") {
                                                    None => {
                                                        //错误的字面量类型
                                                        return token_stream_with_error(item.to_token_stream(),
                                                                                       syn::Error::new_spanned(&item.ident,
                                                                                                               format!("Parse c-like enum failed, val: {}, reason: require integer literal",
                                                                                                                       val)));
                                                    },
                                                    Some(part) => i32::from_str_radix(part, 16).unwrap(),
                                                }
                                            },
                                            Some(part) => i32::from_str_radix(part, 8).unwrap(),
                                        }
                                    },
                                    Some(part) => i32::from_str_radix(part, 2).unwrap(),
                                }
                            },
                            Ok(v) => {
                              v
                            },
                        };
                        let key = stack.pop();
                        (key.unwrap(), value)
                    },
                    any => {
                        //错误的堆栈长度
                        return token_stream_with_error(item.to_token_stream(),
                                                       syn::Error::new_spanned(&item.ident,
                                                                               format!("Parse c-like enum failed, len: {:?}, reason: invalid stack length",
                                                                                       any)));
                    },
                };
                buf.push_back((key, value)); //加入枚举缓冲
            },
            TokenTree::Group(group) => {
                if let Delimiter::Parenthesis = group.delimiter() {
                    //不是类C枚举
                    is_c_like_enum = false;
                }
            },
        }
    }

    if (variants_len == variants_punct_len + 1) && (stack.len() > 0) {
        //处理枚举最后一个成员未以","结束的情况
        let (key, value) = match stack.len() {
            1 => {
                if let Some((_key, last_value)) = buf.back() {
                    //无字面值，则获取上一个枚举成员的值加1，并设置为当前成员的值
                    match (*last_value).checked_add(1) {
                        None => {
                            //越界，则立即返回错误原因
                            return token_stream_with_error(item.to_token_stream(),
                                                           syn::Error::new_spanned(&item.ident,
                                                                                   format!("Parse c-like enum failed, last_value: {:?}, reason: integer overflow",
                                                                                           last_value)));
                        },
                        Some(current_value) => {
                            (stack.pop().unwrap(), current_value)
                        },
                    }
                } else {
                    //无字面值，且没有上一个枚举成员的值
                    (stack.pop().unwrap(), 0)
                }
            },
            2 => {
                //有字面值，则赋值
                let val = stack
                    .pop()
                    .unwrap()
                    .replace("_", "");
                let value = match val.parse::<i32>() {
                    Err(_) => {
                        match val.strip_prefix("0b") {
                            None => {
                                match val.strip_prefix("0o") {
                                    None => {
                                        match val.strip_prefix("0x") {
                                            None => {
                                                //错误的字面量类型
                                                return token_stream_with_error(item.to_token_stream(),
                                                                               syn::Error::new_spanned(&item.ident,
                                                                                                       format!("Parse c-like enum failed, val: {}, reason: require integer literal",
                                                                                                               val)));
                                            },
                                            Some(part) => i32::from_str_radix(part, 16).unwrap(),
                                        }
                                    },
                                    Some(part) => i32::from_str_radix(part, 8).unwrap(),
                                }
                            },
                            Some(part) => i32::from_str_radix(part, 2).unwrap(),
                        }
                    },
                    Ok(v) => {
                        v
                    },
                };
                let key = stack.pop();
                (key.unwrap(), value)
            },
            any => {
                //错误的堆栈长度
                return token_stream_with_error(item.to_token_stream(),
                                               syn::Error::new_spanned(&item.ident,
                                                                       format!("Parse c-like enum failed, len: {:?}, reason: invalid stack length",
                                                                               any)));
            },
        };
        buf.push_back((key, value)); //加入枚举缓冲
    }

    if !is_c_like_enum {
        //不是类C枚举，则立即返回错误原因
        return token_stream_with_error(item.to_token_stream(),
                                       syn::Error::new_spanned(&item.ident,
                                                               format!("Parse c-like enum failed, reason: require c-like enum")));
    }

    let (keys, values) = Vec::from(buf).into_iter().unzip();
    generate_impl_from_i32_to_c_like_enum(item, args, keys, values)
}

// 为类C枚举生成实现From<i32>的代码
fn generate_impl_from_i32_to_c_like_enum(item: ItemEnum,
                                         _args: AttributeArgs,
                                         keys: Vec<String>,
                                         values: Vec<i32>) -> TokenStream2 {
    let item_span = item.span();
    let enum_name = item.ident.clone();

    let idents: Vec<Ident> = keys
        .iter()
        .map(|key| {
            Ident::new(key.as_str(), item_span)
        })
        .collect();

    quote_spanned! {item_span=>
        #item
        impl From<i32> for #enum_name {
            fn from(value: i32) -> Self {
                match value {
                    #(#values => #enum_name::#idents,)*
                    _ => unimplemented!(),
                }
            }
        }
        impl From<#enum_name> for i32 {
            fn from(value: #enum_name) -> Self {
                match value {
                    #(#enum_name::#idents => #values,)*
                }
            }
        }
    }
}

// 构建指定错误的词条流
fn token_stream_with_error(mut tokens: TokenStream2,
                           error: syn::Error) -> TokenStream2 {
    tokens.extend(error.into_compile_error());
    tokens
}