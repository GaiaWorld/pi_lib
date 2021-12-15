extern crate proc_macro;

use std::fs;
use std::path::PathBuf;
use proc_macro::{TokenStream, TokenTree};
use proc_macro2::TokenTree as _TokenTree;

use syn;
use heck::AsLowerCamelCase;

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
            if let syn::Item::Struct(item) = &ast.items[0] {
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
}

#[test]
fn test_snake_case_to_lower_camel_case() {
    let snake_case = "test_get_string";
    let lower_camel_case = format!("{}", AsLowerCamelCase(snake_case));
    println!("{}, {}", snake_case, lower_camel_case);
}
