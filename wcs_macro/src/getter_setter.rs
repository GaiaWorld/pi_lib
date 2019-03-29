/// 为结构体实现getter，setter方法

use quote::{quote};
// use quote::{quote, ToTokens};
use data::*;

pub fn impl_getter_setter_macro(ast: &syn::DeriveInput) -> quote::__rt::TokenStream {
    let name = &ast.ident;
    match &ast.data {
        syn::Data::Struct(s) => {
            let fields = match &s.fields {
                syn::Fields::Named(f) => {
                    Fields::from(&f.named, FieldsType::Named, field_filter)
                },
                syn::Fields::Unnamed(f) => {
                    Fields::from(&f.unnamed, FieldsType::Unnamed, field_filter)
                },
                syn::Fields::Unit => return quote! {},
            };
            impl_struct_set_get(name, &fields)
        },
        syn::Data::Enum(_) => quote! {},
        syn::Data::Union(_) => panic!("cant not impl 'Geter' and 'Seter' for Union"),
    }
}

fn impl_struct_set_get(struct_name: &syn::Ident, feilds: &Fields) -> quote::__rt::TokenStream {
    let Fields{ty, data} = feilds;
    let mut impls = Vec::new();
    match ty {
        FieldsType::Named => {
            for field in data.iter(){
                impls.push(field_set_get(&field.key, &field.ty));
            }
        }
        FieldsType::Unnamed => {
            for i in 0..data.len(){
                let field = &data[i];
                impls.push(field_set_get(&i, &field.ty));
            }
        }
    }
    quote! {
        impl #struct_name{
            #(#impls)*
        }
    }
}

fn field_set_get<T: quote::ToTokens + ToString>(name: &T, ty: &syn::Type) -> quote::__rt::TokenStream {
    let name_str = name.to_string();
    let set_name = set_name(&name_str);
    let get_name = get_name(&name_str);
    let get_name_mut = get_name_mut(&name_str);
    quote! {
        #[inline]
        pub fn #set_name(&mut self, value: #ty){
            self.#name = value;
        }

        #[inline]
        pub fn #get_name(&self) -> &#ty{
            &self.#name
        }

        #[inline]
        pub fn #get_name_mut(&mut self) -> &mut #ty{
            &mut self.#name
        }
    }
}

fn field_filter(_field: &syn::Field, _i: usize) -> bool{
    // let attrs = &field.attrs;
    // for a in attrs.iter(){
    //     if a.path.clone().into_token_stream().to_string().as_str() == "get_set" {
    //         return true;
    //     }
    // }
    true
}