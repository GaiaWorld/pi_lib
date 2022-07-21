extern crate proc_macro;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use proc_macro::{TokenStream, TokenTree};
use proc_macro2::TokenTree as _TokenTree;

use syn;
#[cfg(feature = "ts_lower_camel_case")]
use heck::AsLowerCamelCase;
use syn::__private::ToTokens;

#[test]
fn test_parse_ast() {
    if let Ok(root) = fs::read_dir("./tests/src") {
        let src_root = PathBuf::from("./tests/src");
        let dst_root = PathBuf::from("./tests/dst");
        let mut iter = root.into_iter();

        while let Some(r) = iter.next() {
            if let Ok(file) = r {
                if let Some(filename) = file.file_name().to_str() {
                    let src_file = src_root.join(filename);
                    if let Ok(source) = fs::read_to_string(&src_file) {
                        if let Ok(ast) = syn::parse_file(&source) {
                            let mut dst_file = dst_root.join(filename);
                            dst_file.set_extension("ast");
                            println!("!!!!!!parse {:?} to ast ok", &src_file);
                            let ast_str = format!("{:#?}", ast);
                            let _ = fs::write(&dst_file, ast_str);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_parse_attribute() {
    if let Ok(source) = fs::read_to_string("./tests/src/_9.rs") {
        if let Ok(ast) = syn::parse_file(&source) {
            for item in &ast.items {
                match item {
                    syn::Item::Mod(module) => {
                        if let Some((_, sub_items)) = &module.content {
                            for item in sub_items {
                                if let syn::Item::Struct(item) = item {
                                    for attr in &item.attrs {
                                        match attr.parse_meta() {
                                            Ok(syn::Meta::Path(path)) => {
                                                println!("!!!!!!path ident: {:?}", path.get_ident());
                                            },
                                            Ok(syn::Meta::List(list)) => {
                                                println!("!!!!!!list path ident: {:?}", list.path.get_ident());
                                            },
                                            Ok(syn::Meta::NameValue(kv)) => {
                                                println!("!!!!!!kv path ident: {:?}", kv.path.get_ident());
                                            },
                                            _ => {
                                                for seg in attr.path.segments.iter() {
                                                    println!("!!!!!!path segments ident: {:?}", seg.ident)
                                                }
                                            }
                                        }

                                        for token in attr.tokens.clone() {
                                            match token {
                                                _TokenTree::Punct(punct) => {
                                                    println!("!!!!!!punct: {:?}", punct);
                                                },
                                                _TokenTree::Ident(ident) => {
                                                    println!("!!!!!!ident: {:?}", ident);
                                                },
                                                _TokenTree::Literal(literal) => {
                                                    println!("!!!!!!literal: {:?}", literal);
                                                },
                                                _TokenTree::Group(group) => {
                                                    println!("!!!!!!group: {:?}", group);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    syn::Item::Struct(item) => {
                        for attr in &item.attrs {
                            match attr.parse_meta() {
                                Ok(syn::Meta::Path(path)) => {
                                    println!("!!!!!!path ident: {:?}", path.get_ident());
                                },
                                Ok(syn::Meta::List(list)) => {
                                    println!("!!!!!!list path ident: {:?}", list.path.get_ident());
                                },
                                Ok(syn::Meta::NameValue(kv)) => {
                                    println!("!!!!!!kv path ident: {:?}", kv.path.get_ident());
                                },
                                _ => {
                                    for seg in attr.path.segments.iter() {
                                        println!("!!!!!!path segments ident: {:?}", seg.ident)
                                    }
                                }
                            }

                            for token in attr.tokens.clone() {
                                match token {
                                    _TokenTree::Punct(punct) => {
                                        println!("!!!!!!punct: {:?}", punct);
                                    },
                                    _TokenTree::Ident(ident) => {
                                        println!("!!!!!!ident: {:?}", ident);
                                    },
                                    _TokenTree::Literal(literal) => {
                                        println!("!!!!!!literal: {:?}", literal);
                                    },
                                    _TokenTree::Group(group) => {
                                        println!("!!!!!!group: {:?}", group);
                                    }
                                }
                            }
                        }
                    },
                    _ => (),
                }
            }
        }
    }
}

#[cfg(feature = "ts_lower_camel_case")]
#[test]
fn test_snake_case_to_lower_camel_case() {
    let snake_case = "test_get_string";
    let lower_camel_case = format!("{}", AsLowerCamelCase(snake_case));
    println!("{}, {}", snake_case, lower_camel_case);
}
