#![feature(custom_attribute)]
#![recursion_limit="512"]
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
    let mut adds_with_context = Vec::new();
    let mut dels = Vec::new();
    let mut res = Vec::new();
    let mut res_new = Vec::new();
    let mut single = Vec::new();
    let mut single_sets = Vec::new();
    let mut single_gets = Vec::new();
    let mut single_gets_mut = Vec::new();
    let mut single_tys = Vec::new();
    let mut single_mgrs = Vec::new();
    let mut single_names = Vec::new();
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
            dels.push(del_name(&field_name_str));
            adds_with_context.push(add_with_context_name(&field_name_str));
        } else if is_single_component(field){
            let field_name_str = match &field.ident {
                Some(ref i) => i.to_string(),
                None => panic!("no fieldname"),
            };
            let name = &field.ident;
            let ty = &field.ty;
            single.push(quote!{
                pub #name: SingleCase<#ty, Self>,
            });
            single_gets.push(get_name(&field_name_str));
            single_gets_mut.push(get_name_mut(&field_name_str));
            single_sets.push(set_name(&field_name_str));
            single_tys.push(ty.clone());
            single_mgrs.push(ident(&mgr_str));
            single_names.push(name.clone());
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

    let field_names1 = &field_names;
    let field_names2 = &field_names;
    let field_names3 = &field_names;
    let field_names4 = &field_names;
    let field_names5 = &field_names;
    let field_names6 = &field_names;
    let field_names7 = &field_names;
    let field_names8 = &field_names;
    let field_types_c1 = &field_types_c;
    let field_types_c2 = &field_types_c;
    let mgrs1 = &mgrs;
    let mgrs2 = &mgrs;
    let mgrs3 = &mgrs;
    let mgrs4 = &mgrs;
    let mgrs5 = &mgrs;
    let read_refs1 = &read_refs;
    let read_refs2 = &read_refs;
    let write_refs1 = &write_refs;
    let write_refs2 = &write_refs;
    let write_refs3 = &write_refs;
    let write_refs4 = &write_refs;
    let write_refs5 = &write_refs;
    let write_refs6 = &write_refs;
    let write_refs7 = &write_refs;

    let single_mgrs1 = &single_mgrs;
    let single_mgrs2 = &single_mgrs;
    let single_tys1 = &single_tys;
    let single_tys2 = &single_tys;
    let single_names1 = &single_names;
    let single_names2 = &single_names;

    let gen = quote! {
        pub struct #mgr_name{
            #(#res)*
            #(#single)*
            #(pub #field_names1: #field_groups<#mgrs1>),*
        }

        impl ComponentMgr for #mgr_name{
            // fn new() -> Self{
            //     #mgr_name{
            //         #(#res_new)*
            //         #(#field_names1: #field_groups1::new()),*
            //     }
            // }
        }

        impl #mgr_name {
            #(
                pub fn #adds(&mut self, value: #field_types_c1) -> #write_refs1<#mgrs2>{
                    let id = self.#field_names2._group.insert(value, 0);
                    let mut r = #write_refs2::new(id, self.#field_names3.to_usize(), self);
                    r.set_parent(0);
                    r.create_notify();
                    r
                }

                pub fn #dels(&mut self, id: usize){
                    let mut r = #write_refs7::new(id, self.#field_names8.to_usize(), self);
                    r.destroy();
                }

                pub fn #adds_with_context(&mut self, value: #field_types_c2, context: usize) -> #write_refs3<#mgrs3>{
                    let id = self.#field_names4._group.insert(value, context);
                    let mut r = #write_refs4::new(id, self.#field_names5.to_usize(), self);
                    r.set_parent(context);
                    r.create_notify();
                    r
                }

                pub fn #field_gets(&mut self, index: usize) -> #read_refs1<#mgrs4>{
                    #read_refs2::new(index, &self.#field_names6)
                }

                pub fn #field_gets_mut(&mut self, index: usize) -> #write_refs5<#mgrs5>{
                    #write_refs6::new(index.clone(), self.#field_names7.to_usize(), self)
                }
            )*

            #(
                pub fn #single_gets(&mut self) -> &#single_tys1{
                    &self.#single_names1
                }

                pub fn #single_gets_mut(&mut self) -> SingleCaseWriteRef<#single_tys2, #single_mgrs2>{
                    let mgr = self as *const #single_mgrs1 as usize;
                    SingleCaseWriteRef::new(&mut self.#single_names2, mgr)
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






