extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream, Result},
    DeriveInput, Path,
};

// pub trait System {
//     fn fetch_setup(me: Arc<Any>, world: &World) -> Option<RunnerFn> where Self: Sized;
//     fn fetch_run(me: Arc<Any>, world: &World) -> Option<RunnerFn> where Self: Sized;
//     fn fetch_dispose(me: Arc<Any>, world: &World) -> Option<RunnerFn> where Self: Sized;
// }

// #[proc_macro_derive(System, attributes(storage))]
// pub fn component(input: TokenStream) -> TokenStream {
//     let ast = syn::parse(input).unwrap();
//     let gen = impl_component(&ast);
//     gen.into()
// }

#[proc_macro_derive(Component, attributes(storage))]
pub fn component(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_component(&ast);
    gen.into()
}

fn impl_component(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let storage = ast
        .attrs
        .iter()
        .find(|attr| attr.path.segments[0].ident == "storage")
        .map(|attr| {
            syn::parse2::<StorageAttribute>(attr.tts.clone())
                .unwrap()
                .storage
        })
        .unwrap_or_else(|| parse_quote!(DenseVecStorage));

    quote! {
        impl #impl_generics Component for #name #ty_generics #where_clause {
            type Storage = #storage<Self>;
        }
    }
}