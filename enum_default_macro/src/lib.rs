//! 提供一个宏，该宏可以为枚举类型实现`std::default::Default`
//! 实现`std::default::Default`是，是将枚举的第一个类型作为默认值
//! 
//! # example
//! 
//! derive(EnumDefault, Debug)
//! enum AA {
//!     A,
//!     B,
//!     C,
//! }
//! println!("AA default:{:?}", AA::default());//AA::A为默认值
//!
//! # example
//!
//! derive(EnumDefault, Debug)
//! enum BB {
//!     A{id:number},
//!     B,
//!     C,
//! }
//! println!("BB default:{:?}", BB::default());BB::A{id:0}为默认值

#![recursion_limit="256"]
extern crate proc_macro;
extern crate quote;
extern crate syn;
extern crate proc_macro2;

use crate::proc_macro::TokenStream;

use quote::quote;

#[proc_macro_derive(EnumDefault)]
pub fn default_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_default_macro(&ast);
    gen.into()
}

fn impl_default_macro(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    match &ast.data {
        syn::Data::Struct(_) => panic!("it's not a enum"),
        syn::Data::Enum(e) => {
            enum_default(name, &e.variants)
        },
        syn::Data::Union(_) => panic!("it's not a enum"),
    }
}

fn enum_default(name: &syn::Ident, variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> proc_macro2::TokenStream{
    if variants.len() == 0 {
        panic!("impl_default error");
    }
    
    let first_variant = match variants.first() {
        Some(v) => v,
        None => panic!("enum variants len is 0"),
    };
    let first_variant_name = &first_variant.ident;
    let f = variant_default(&first_variant.fields);
    let f:proc_macro2::TokenStream = f.into();
    quote!{
        impl std::default::Default for #name {
            fn default() -> #name{
                #name::#first_variant_name#f.into()
            }
        }
    }
}

fn variant_default(fields: &syn::Fields) -> proc_macro2::TokenStream{
    let mut is_named = false;
    let fields = match fields {
        syn::Fields::Named(named) => {is_named = true; &named.named},
        syn::Fields::Unnamed(unnamed) => &unnamed.unnamed,
        syn::Fields::Unit => return quote!{}.into(),
    };

    let mut arr = Vec::new();
    
    if is_named {
        for field in fields.iter(){
            let name = field.ident.clone().unwrap();
            let ty = &field.ty;
            arr.push(quote!{#name: <#ty>::default()});
        }
        return quote!{{#(#arr),*};
        };
    }else {
        for field in fields.iter(){
            let ty = &field.ty;
            arr.push(quote!{<#ty>::default()});
        }
        return quote!{(#(#arr),*)};
    }
}









