#![feature(custom_attribute)]
#![recursion_limit="256"]
extern crate proc_macro;
extern crate quote;
extern crate syn;

use crate::proc_macro::TokenStream;

use quote::quote;

#[proc_macro_derive(EnumDefault)]
pub fn default_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_default_macro(&ast);
    gen.into()
}

fn impl_default_macro(ast: &syn::DeriveInput) -> quote::__rt::TokenStream {
    let name = &ast.ident;
    match &ast.data {
        syn::Data::Struct(_) => panic!("it's not a enum"),
        syn::Data::Enum(e) => {
            enum_default(name, &e.variants)
        },
        syn::Data::Union(_) => panic!("it's not a enum"),
    }
}

fn enum_default(name: &syn::Ident, variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> quote::__rt::TokenStream{
    if variants.len() == 0 {
        panic!("impl_default error");
    }
    
    let first_variant = match variants.first() {
        Some(v) => match v {
            syn::punctuated::Pair::Punctuated(v, _p) => v,
            syn::punctuated::Pair::End(v) => v,
        },
        None => panic!("enum variants len is 0"),
    };
    let first_variant_name = &first_variant.ident;
    quote!{
        impl std::default::Default for #name {
            fn default() -> #name{
                #name::#first_variant_name
            }
        }
    }
}









