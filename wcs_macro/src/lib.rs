#![recursion_limit="128"]
extern crate slab;
extern crate wcs;
extern crate proc_macro;
extern crate quote;
extern crate syn;

mod util;
mod enum_component;
mod component;

use crate::proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use enum_component::*;
use component::*;
use util::*;

#[proc_macro_derive(Component)]
pub fn component_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_component_macro(&ast)
}

#[proc_macro]
pub fn out_component(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_component_macro(&ast)
}

#[proc_macro_derive(EnumComponent)]
pub fn ennum_component_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_enum_component_macro(&ast)
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
    let mut field_types = Vec::new();
    let mut mgrs = Vec::new();
    let mut refs = Vec::new();
    let mut adds = Vec::new();
    for field in fields.iter(){
        let field_name_str = match &field.ident {
            Some(ref i) => i.to_string(),
            None => panic!("no fieldname"),
        };
        let field_ty = field.ty.clone().into_token_stream().to_string();
        field_names.push(ident(&field_name_str));
        field_groups.push(group_name(field_ty.clone()));
        field_types.push(ident(&field_ty));
        mgrs.push(ident(&mgr_str));
        refs.push(ref_name(field_ty));
        adds.push(add_name(&field_name_str));
    }

    let field_names1 = field_names.clone();
    let field_groups1 = field_groups.clone();
    let field_names2 = field_names.clone();
    let field_names4 = field_names.clone();
    let field_names5 = field_names.clone();
    let field_names6 = field_names.clone();
    let field_names7 = field_names.clone();
    let mgrs1 = mgrs.clone();
    let refs1 = refs.clone();

    let gen = quote! {
        pub struct #mgr_name{
            #(pub #field_names: Rc<RefCell<#field_groups<#mgrs>>>),*
        }

        impl ComponentMgr for #mgr_name{
            fn new() -> Rc<RefCell<Self>>{
                let m = Rc::new(RefCell::new(#mgr_name{
                    #(#field_names1: Rc::new(RefCell::new(#field_groups1::new()))),*
                }));
                let m_weak = Rc::downgrade(&m);
                {
                    let m_borrow = m.borrow();
                    #(m_borrow.#field_names2.borrow_mut().set_mgr(m_weak.clone());)*
                }
                m
            }
        }

        impl #mgr_name {
            #(
                pub fn #adds(&mut self, #field_names4: #field_types) -> #refs<#mgrs1>{
                    let point = self.#field_names6.borrow_mut()._group.insert(#field_names7, 0);
                    #refs1::new(point, self.#field_names5.clone())
                }
            )*
        }
    };
    gen.into()
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






