#![feature(custom_attribute)]
#![recursion_limit="256"]
extern crate slab;
extern crate wcs;
extern crate proc_macro;
extern crate quote;
extern crate syn;

mod data;
mod enum_component;
mod component;
mod getter_setter;
mod builder;

use crate::proc_macro::TokenStream;

use quote::quote;
use quote::ToTokens;
use enum_component::*;
use component::*;
use data::*;
use getter_setter::*;
use builder::*;

#[proc_macro_derive(Component)]
pub fn component_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let mut arr = Vec::new();
    arr.push(impl_getter_setter_macro(&ast));
    arr.push(impl_component_macro(&ast)); 
    let gen = quote! {
        #(#arr)*
    };
    gen.into()
}

#[proc_macro]
pub fn getter_setter(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let gen = impl_getter_setter_macro(&ast);
    gen.into()
}

#[proc_macro]
// #[proc_macro_derive(Component)]
pub fn component(input: TokenStream) -> TokenStream {
    // let attr_str = attrs.to_string();
    // et mut attrs = HashMap::new();
    // for s in attr_str.split(","){
    //     attrs.insert(s.to_string(), true);
    // }
    
    let ast = syn::parse(input).unwrap();
    impl_component_macro(&ast).into()
}

#[proc_macro_derive(EnumComponent)]
pub fn ennum_component_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    match ast.data {
        syn::Data::Enum(data) => impl_enum_component_macro(&EnumData::from(&data, &ast.ident)),
        _ => panic!("enum errorQ"),
    }
}

#[proc_macro]
pub fn world(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let mgr_name = &ast.ident;
    let mgr_str = mgr_name.to_string();
    let fields = match &ast.data {
        syn::Data::Struct(ref s) => {
            match &s.fields {
                syn::Fields::Named(f) => {
                    &f.named
                },
                _ => panic!("feild error, it must is struct"),
            }
        },
        _ => panic!("paream error, it must is struct"),
    };

    let mut field_names = Vec::new();
    let mut field_groups = Vec::new();
    let mut field_types_c = Vec::new();
    let mut field_names_c = Vec::new();
    let mut field_types_enum_c = Vec::new();
    let mut field_names_enum_c = Vec::new();
    let mut field_ids = Vec::new();
    let mut field_gets = Vec::new();
    let mut field_gets_mut = Vec::new();
    let mut mgrs = Vec::new();
    let mut read_refs = Vec::new();
    let mut write_refs = Vec::new();
    let mut adds = Vec::new();
    let mut res = Vec::new();
    let mut res_new = Vec::new();
    for field in fields.iter(){
        if is_component(&field) || is_enum_component(&field)  {
            let field_name_str = match &field.ident {
                Some(ref i) => i.to_string(),
                None => panic!("no fieldname"),
            };
            let field_ty = field.ty.clone().into_token_stream().to_string();
            field_names.push(ident(&field_name_str));
            field_groups.push(group_name(field_ty.clone()));
            if is_component(&field){
                field_types_c.push(ident(&field_ty));
                field_names_c.push(ident(&field_name_str));
            }else {
                field_types_enum_c.push(ident(&field_ty));
                field_names_enum_c.push(ident(&field_name_str));
            }
            
            field_ids.push(id_name(field_ty.clone()));
            field_gets.push(get_name(&field_name_str));
            field_gets_mut.push(get_name_mut(&field_name_str));
            mgrs.push(ident(&mgr_str));
            read_refs.push(read_ref_name(field_ty.clone()));
            write_refs.push(write_ref_name(field_ty));
            adds.push(add_name(&field_name_str));
        }else {
            let name = &field.ident;
            let ty = &field.ty;
            res.push(quote!{
                pub #name: #ty,
            });
            let mut field_ty = field.ty.clone();
            match &mut field_ty {
                syn::Type::Path(ref mut p) => {
                    for v in p.path.segments.iter_mut(){
                        v.arguments = syn::PathArguments::None;
                    }
                },
                _ => panic!("type error"),
            }
            res_new.push(quote!{
                #name: #field_ty::new(),
            })
        }
        
    }

    let field_names1 = field_names.clone();
    let field_names2 = field_names.clone();
    let field_groups1 = field_groups.clone();
    let field_names8 = field_names.clone();
    let field_names9 = field_names.clone();
    let mgrs1 = mgrs.clone();
    let read_refs1 = read_refs.clone();
    let mgrs2 = mgrs.clone();
    let mgrs3 = mgrs.clone();
    let read_refs2 = read_refs.clone();
    let write_refs1 = write_refs.clone();
    let write_refs2 = write_refs.clone();
    let write_refs3 = write_refs.clone();

    let gen = quote! {
        pub struct #mgr_name{
            #(#res)*
            #(pub #field_names: #field_groups<#mgrs>),*
        }

        impl ComponentMgr for #mgr_name{
            fn new() -> Self{
                #mgr_name{
                    #(#res_new)*
                    #(#field_names1: #field_groups1::new()),*
                }
            }
        }

        impl #mgr_name {
            #(
                // pub fn #adds(&mut self, #field_names4: #field_types) -> #refs<#mgrs1>{
                //     let id = self.#field_names6.borrow_mut()._group.insert(#field_names7, 0);
                //     #refs1::new(id, self.#field_names5.clone())
                // }
                pub fn #adds(&mut self, value: #field_types_c) -> #write_refs<#mgrs1>{
                    let id = self.#field_names_c._group.insert(value, 0);
                    let mut r = #write_refs1::new(id, self.#field_names2.to_usize(), self);
                    r.set_parent(0);
                    r.create_notify();
                    r
                    // #write_refs1::create(parent, self.#field_names5.to_usize(), self)
                }

                pub fn #field_gets(&mut self, index: usize) -> #read_refs1<#mgrs2>{
                    #read_refs2::new(index, &self.#field_names8)
                }

                pub fn #field_gets_mut(&mut self, index: usize) -> #write_refs2<#mgrs3>{
                    #write_refs3::new(index.clone(), self.#field_names9.to_usize(), self)
                }
            )*
        }
    };
    gen.into()
}

#[proc_macro_derive(Builder)]
pub fn builder_macro_derive(item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    impl_builder_macro(&ast).into()
}

#[proc_macro]
pub fn out_component(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_component_macro(&ast).into()
}

// fn impl_enum_component_macro(ast: &syn::DeriveInput) -> TokenStream {
//     let name = &ast.ident;
//     let gen = match &ast.data {
//         syn::Data::Enum(s) => {
//             let variants = &s.variants;
//             for v in variants.iter() {

//             }
//         },
//         syn::Data::Union(s) => panic!("must is enum"),
//     };
//     gen.into()
// }

// fn impl_enum_component() -> TokenStream{

// }






