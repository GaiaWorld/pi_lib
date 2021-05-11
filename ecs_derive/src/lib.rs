extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream, Result},
    DeriveInput, Path,
};

/// Custom derive macro for the `Component` trait.
///
/// ## Example
///
/// ```rust,ignore
/// extern crate map;
/// use map::VecMap;
///
/// #[derive(Component, Debug)]
/// #[storage(VecMap)] //  `VecMap` is a data structure for a storage component, This line is optional, defaults to `VecMap`
/// struct Pos(f32, f32, f32);
/// ```
#[proc_macro_derive(Component, attributes(storage))]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_component(&ast, false);
    gen.into()
}

#[proc_macro_derive(Write)]
pub fn write_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_write(&ast, &ast.generics, false);
    gen.into()
}

#[proc_macro]
pub fn write(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_component(&ast, false);
    gen.into()
}

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_component(&ast, true);
    gen.into()
}

struct StorageAttribute {
    storage: Path,
}

impl Parse for StorageAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _parenthesized_token = parenthesized!(content in input);

        Ok(StorageAttribute {
            storage: content.parse()?,
        })
    }
}

fn impl_component(ast: &DeriveInput, is_deref: bool) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let storage = ast
        .attrs
        .iter()
        .find(|attr| attr.path.segments[0].ident == "storage")
        .map(|attr| {
            syn::parse2::<StorageAttribute>(attr.tokens.clone())
                .unwrap()
                .storage
        })
        .unwrap_or_else(|| parse_quote!(VecMap));

    let write: proc_macro2::TokenStream = impl_write(ast, &ast.generics, is_deref).into();

    quote! {
        impl #impl_generics Component for #name #ty_generics #where_clause {
            type Storage = #storage<Self>;
        }

        #write
    }
}

fn impl_write(ast: &DeriveInput, generics: &syn::Generics, is_deref: bool) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let write_trait_name = ident(&(name.to_string() + "Write"));
    let trait_def = SetGetFuncs(ast);
    let trait_impl = SetGetFuncsImpl(ast, is_deref);

    let mut generics1 = generics.clone();
    generics1.params.insert(0, syn::GenericParam::Lifetime(syn::LifetimeDef::new(syn::Lifetime::new("'a", proc_macro2::Span::call_site()))));
    let (trait_generics, ty_generics, where_clause) = generics.split_for_impl();
    let (impl_generics, _, _) = generics1.split_for_impl();

    quote! {
        pub trait #write_trait_name#trait_generics #where_clause {
            #trait_def
        }

        impl#impl_generics #write_trait_name#ty_generics for ecs::monitor::Write<'a, #name #ty_generics> #where_clause {
            #trait_impl
        }
    }
}


fn ident(sym: &str) -> syn::Ident {
    syn::Ident::new(sym, proc_macro2::Span::call_site())
}

struct SetGetFuncsImpl<'a>(&'a syn::DeriveInput, bool);

impl<'a> ToTokens for SetGetFuncsImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.0.ident;
        let (_, ty_generics, _) = self.0.generics.split_for_impl();
        match &self.0.data {
            syn::Data::Struct(s) => {
                let fields = &s.fields;
                match fields {
                    syn::Fields::Named(fields) => {
                        for field in fields.named.iter() {
                            let field_name = field.ident.as_ref().unwrap();
                            let field_name_str = field_name.clone().to_string();
                            let set_name = ident(&("set_".to_string() + field_name.clone().to_string().as_str()));
                            let ty = &field.ty;
                            if is_base_type(ty) {
                                // set field
                                if self.1 {
                                    tokens.extend(quote! {
                                        fn #set_name(&mut self, value: #ty) {
                                            if value == (self.value.0).#field_name {
                                                return;
                                            }
                                            (self.value.0).#field_name = value;
                                            self.notify.modify_event(self.id, #field_name_str, 0);
                                        } 
                                    });
                                }else {
                                    tokens.extend(quote! {
                                        fn #set_name(&mut self, value: #ty) {
                                            if value == self.value.#field_name {
                                                return;
                                            }
                                            self.value.#field_name = value;
                                            self.notify.modify_event(self.id, #field_name_str, 0);
                                        } 
                                    });
                                }
                            } else {
                                // set field
                                if self.1 {
                                    tokens.extend(quote! {
                                        fn #set_name(&mut self, value: #ty) {
                                            (self.value.0).#field_name = value;
                                            self.notify.modify_event(self.id, #field_name_str, 0);
                                        } 
                                    });
                                }else {
                                    tokens.extend(quote! {
                                        fn #set_name(&mut self, value: #ty) {
                                            self.value.#field_name = value; 
                                            self.notify.modify_event(self.id, #field_name_str, 0);
                                        } 
                                    });
                                }
                            }
                        }
                    },
                    syn::Fields::Unnamed(fields) => {
                        let mut i: usize = 0;
                        for field in fields.unnamed.iter() {
                            let set_name = ident(&("set_".to_string() + i.to_string().as_str()));
                            let ty = &field.ty;
                            let index = syn::Index::from(i);
                            if is_base_type(ty) {
                                // set index
                                if self.1 {
                                    tokens.extend(quote! {
                                        fn #set_name(&mut self, value: #ty) {
                                            if (self.value.0).#index == value {
                                                return;
                                            }
                                            (self.value.0).#index = value;
                                            self.notify.modify_event(self.id, "", #i);
                                        } 
                                    });
                                }else {
                                    tokens.extend(quote! {
                                        fn #set_name(&mut self, value: #ty) {
                                            if self.value.#index == value {
                                                return;
                                            }
                                            self.value.#index = value;
                                            self.notify.modify_event(self.id, "", #i);
                                        } 
                                    });
                                }
                            }else {
                                // set index
                                if self.1 {
                                    tokens.extend(quote! {
                                        fn #set_name(&mut self, value: #ty) {
                                            (self.value.0).#index = value;
                                            self.notify.modify_event(self.id, "", #i);
                                        } 
                                    });
                                }else {
                                    tokens.extend(quote! {
                                        fn #set_name(&mut self, value: #ty) {
                                            self.value.#index = value;
                                            self.notify.modify_event(self.id, "", #i);
                                        } 
                                    });
                                }
                            }
                            i += 1;
                        }
                    },
                    syn::Fields::Unit => (),
                };
            },
            _ => ()
        };
        // modify
        tokens.extend(quote! {
            fn modify<F: FnOnce(&mut #name#ty_generics) -> bool>(&mut self, callback: F) {
                if callback(self.value) {
                    self.notify.modify_event(self.id, "", 0);
                }
            } 
        });
    }
}

struct SetGetFuncs<'a>(&'a syn::DeriveInput);

impl<'a> ToTokens for SetGetFuncs<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.0.ident;
        let (_, ty_generics, _) = self.0.generics.split_for_impl();
        match &self.0.data {
            syn::Data::Struct(s) => {
                let fields = &s.fields;
                match fields {
                    syn::Fields::Named(fields) => {
                        for field in fields.named.iter() {
                            let field_name = field.ident.as_ref().unwrap();
                            let set_name = ident(&("set_".to_string() + field_name.clone().to_string().as_str()));
                            let ty = &field.ty;
                            // set field def
                            tokens.extend(quote! {
                                fn #set_name(&mut self, ty: #ty);
                            });
                        }
                    },
                    syn::Fields::Unnamed(fields) => {
                        let mut i: usize = 0;
                        for field in fields.unnamed.iter() {
                            let set_name = ident(&("set_".to_string() + i.to_string().as_str()));
                            let ty = &field.ty;
                            // set index def
                            tokens.extend(quote! {
                                fn #set_name(&mut self, ty: #ty);
                            });
                            i += 1;
                        }
                    },
                    syn::Fields::Unit => (),
                };
            },
            _ => ()
        };
        // modify def
        tokens.extend(quote! {
            fn modify<F: FnOnce(&mut #name#ty_generics) -> bool>(&mut self, callback: F); 
        });
    }
}

fn is_base_type(ty: &syn::Type) -> bool{
    let s = ty.into_token_stream().to_string();
    let s = s.as_str();

    if s == "bool" || s == "usize" || s == "isize" || s == "u8" || s == "u16" || s == "u32" || s == "u64" || s == "u128" || s == "i8" || s == "i16" || s == "i32" || s == "i64" || s == "i128" || s == "&str" || s == "String" || s == "&'static str" || s == "f32" || s == "f64" {
        return true;
    }else {
        return false;
    }
}