use quote::{quote, ToTokens};

use data::*;

pub fn impl_builder_macro(ast: &syn::DeriveInput) -> quote::__rt::TokenStream {
    let name = &ast.ident;
    match &ast.data {
        syn::Data::Struct(s) => {
            match &s.fields {
                syn::Fields::Named(ref fields) => struct_builder(name, &fields.named),
                _ => panic!("type error"),
            }
        },
        syn::Data::Enum(e) => {
            enum_builder(name, &e.variants)
        },
        syn::Data::Union(_) => panic!("xxxx"),
    }
}

fn struct_builder(name: &syn::Ident, fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> quote::__rt::TokenStream{
    let mut arr = Vec::new();
    arr.push(def_struct_builder(name, fields));
    arr.push(impl_struct_builder_new(name, fields));
    arr.push(impl_struct_build(name, fields));
    arr.push(impl_struct_field_sets(name, fields));
    quote!{
        #(#arr)*
    }
}

//定义组件的Builder类型
fn def_struct_builder(name: &syn::Ident, fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> quote::__rt::TokenStream{
    let builder_name = builder_name(name.to_string());
    let mut field_def = Vec::new();
    for field in fields.iter() {
        let builder_attrs = paser_builder_attrs(field);
        if !builder_attrs.is_export {
            continue;
        }
        let name = field.ident.clone().unwrap();
        if is_component(field) || is_enum_component(field){
            let ty = ident(&component_name(field));
            field_def.push(quote!{#name: Option<#ty>});
        }else if is_base_type(&field.ty){
            let ty = &field.ty;
            field_def.push(quote!{#name: #ty});
        }else {
            let ty = &field.ty;
            field_def.push(quote!{#name: Option<#ty>});
        }
    }
    quote!{
        pub struct #builder_name{
            #(#field_def),*
        }
    }
}

//定义组件的Builder类型
fn impl_struct_builder_new(name: &syn::Ident, fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> quote::__rt::TokenStream{
    let builder_name = builder_name(name.to_string());
    let mut field_init = Vec::new();
    for field in fields.iter() {
        let builder_attrs = paser_builder_attrs(field);
        if !builder_attrs.is_export {
            continue;
        }
        let name = field.ident.clone().unwrap();
        if !is_component(field) && !is_enum_component(field) && is_base_type(&field.ty){
            let ty = &field.ty;
            field_init.push(quote!{#name: <#ty as std::default::Default>::default()});
        }else {
            field_init.push(quote!{#name: None});
        }
    }
    quote!{
        impl #builder_name {
            pub fn new() -> #builder_name{
                #builder_name{
                    #(#field_init),*
                }
            }
        }
    }
}

//为builder实现build方法
fn impl_struct_build(name: &syn::Ident, fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> quote::__rt::TokenStream{
    let mut field_init = Vec::new();
    for field in fields.iter() {
        let builder_attrs = paser_builder_attrs(field);
        if !builder_attrs.is_export{
            if is_component(field){
                let name = field.ident.clone().unwrap();
                if builder_attrs.build_ty == BuildType::Builder {
                    let ty = ident(&component_name(field));
                    let field_builder_name = builder_name(ty.to_string());
                    field_init.push(quote!{#name: {
                        let v = Builder::build(#field_builder_name::new(), &mut group.#name);
                        group.#name._group.insert(v, 0)
                    }});
                }else if builder_attrs.build_ty == BuildType::Default {
                    let ty = ident(&component_name(field));
                    field_init.push(quote!{#name: {
                        let v = <#ty as std::default::Default>::default();
                        group.#name._group.insert(v, 0)
                    }});
                }else {
                    field_init.push(quote!{#name: 0});
                }
            }else if is_enum_component(field){
                let name = field.ident.clone().unwrap();
                let ty_id_name = id_name(component_name(field));
                if builder_attrs.build_ty == BuildType::Builder {
                    let ty = ident(&component_name(field));
                    let field_builder_name = builder_name(ty.to_string());
                    field_init.push(quote!{#name: {
                        let v = Builder::build(#field_builder_name::new(), &mut group.#name);
                        #ty_id_name::_set(&mut group.#name, v, 0)
                    }});
                }else if builder_attrs.build_ty == BuildType::Default{
                    let ty = ident(&component_name(field));
                    field_init.push(quote!{#name: {
                        let v = <#ty as std::default::Default>::default();
                        group.#name._group.insert(v, 0)
                    }});
                }else {
                    field_init.push(quote!{#name: #ty_id_name::None});
                }
            }else {
                let name = field.ident.clone().unwrap();
                let ty = &field.ty;
                field_init.push(quote!{#name: <#ty as std::default::Default>::default()});
            }
            continue;
        }
        if is_component(field){
            if builder_attrs.build_ty == BuildType::Builder {
                field_init.push(struct_builder_c_field_builder(&field.ident.clone().unwrap(), &ident(&component_name(field))))
            }else if builder_attrs.build_ty == BuildType::Default{
                field_init.push(struct_default_c_field_builder(&field.ident.clone().unwrap(), &ident(&component_name(field))))
            }else {
                field_init.push(struct_c_field_builder(&field.ident.clone().unwrap()))
            }
        }else if is_enum_component(field){
            if builder_attrs.build_ty == BuildType::Builder {
                field_init.push(struct_builder_enum_c_field_builder(&field.ident.clone().unwrap(), &ident(&component_name(field))))
            }else if builder_attrs.build_ty == BuildType::Default{
                field_init.push(struct_defualt_enum_c_field_builder(&field.ident.clone().unwrap(), &ident(&component_name(field))))
            }else {
                field_init.push(struct_enum_c_field_builder(&field.ident.clone().unwrap(), &ident(&component_name(field))))
            }
        }else if is_base_type(&field.ty) {
            let name = field.ident.clone().unwrap();
            field_init.push(quote!{#name: self.#name});
        }else {
            field_init.push(struct_struct_field_builder(&field.ident.clone().unwrap(), &field.ty));
        }
        
    }
    let group_name = group_name(name.to_string());
    let builder_name = builder_name(name.to_string());
    quote!{
        impl<C: ComponentMgr> Builder<C, #group_name<C>, #name> for #builder_name{
            fn build(self, group: &mut #group_name<C>) -> #name{
                #name {
                    #(#field_init),*
                }
            }
        }
    }
}

// 所有字段的设置方法
fn impl_struct_field_sets(name: &syn::Ident, fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> quote::__rt::TokenStream{
    let builder_name = builder_name(name.to_string());
    let mut fns = Vec::new();
    for field in fields.iter() {
        let name = field.ident.clone().unwrap(); //字段名称
        let builder_attrs = paser_builder_attrs(field);
        if !builder_attrs.is_export{
            continue;
        }
        if is_component(field) || is_enum_component(field) {
            fns.push(struct_field_set(&name, &ident(&component_name(field))));
        }else if is_base_type(&field.ty){
            fns.push(struct_base_field_set(&name, &field.ty));
        }
        else {
            fns.push(struct_field_set(&name, &field.ty));
        }
    }

    quote!{
        impl #builder_name{
            #(#fns)*
        }
    }
}

// 每个字段的设置方法(除基础类型的字段)
fn struct_field_set<T: quote::ToTokens>(name: &syn::Ident, ty: &T) -> quote::__rt::TokenStream{
    quote!{
        pub fn #name(mut self, value: #ty) -> Self{
            self.#name = Some(value);
            self
        }
    }
}

// 每个字段的设置方法
fn struct_base_field_set<T: quote::ToTokens>(name: &syn::Ident, ty: &T) -> quote::__rt::TokenStream{
    quote!{
        pub fn #name(mut self, value: #ty) -> Self{
            self.#name = value;
            self
        }
    }
}

// 必须的子组件插入容器并取值， 如果没有， 会创建
fn struct_builder_c_field_builder(name: &syn::Ident, ty: &syn::Ident) -> quote::__rt::TokenStream{
    let field_builder_name = builder_name(ty.to_string());
    quote!{
        #name: match self.#name {
            Some(v) => group.#name._group.insert(v, 0),
            None => {
                let v = Builder::build(#field_builder_name::new(), &mut group.#name);
                group.#name._group.insert(v, 0)
            },
        }
    }
}

// 必须的子组件插入容器并取值， 如果没有， 会创建
fn struct_default_c_field_builder(name: &syn::Ident, ty: &syn::Ident) -> quote::__rt::TokenStream{
    quote!{
        #name: match self.#name {
            Some(v) => group.#name._group.insert(v, 0),
            None => {
                let v = <#ty as std::default::Default>::default();
                group.#name._group.insert(v, 0)
            },
        }
    }
}

// 非必须的子组件插入容器并取值， 如果没有， 返回0
fn struct_c_field_builder(name: &syn::Ident) -> quote::__rt::TokenStream{
    quote!{
        #name: match self.#name {
            Some(v) => group.#name._group.insert(v, 0),
            None => 0,
        }
    }
}

// 必须的子组件插入容器并取值， 如果没有， 会创建(枚举组件)
fn struct_builder_enum_c_field_builder(name: &syn::Ident, ty: &syn::Ident) -> quote::__rt::TokenStream{
    let field_builder_name = builder_name(ty.to_string());
    let field_id_name = id_name(ty.to_string());
    quote!{
        #name: match self.#name {
            Some(v) => #field_id_name::_set(&mut group.#name, v, 0),
            None => {
                let v = Builder::build(#field_builder_name::new(), &mut group.#name);
                #field_id_name::_set(&mut group.#name, v, 0)
            },
        }
    }
}

fn struct_defualt_enum_c_field_builder(name: &syn::Ident, ty: &syn::Ident) -> quote::__rt::TokenStream{
    let field_id_name = id_name(ty.to_string());
    quote!{
        #name: match self.#name {
            Some(v) => #field_id_name::_set(&mut group.#name, v, 0),
            None => {
                let v = <#ty as std::default::Default>::default();
                #field_id_name::_set(&mut group.#name, v, 0)
            },
        }
    }
}

// 非必须的子组件插入容器并取值， 如果没有， 返回Id::None(枚举组件)
fn struct_enum_c_field_builder(name: &syn::Ident, ty: &syn::Ident) -> quote::__rt::TokenStream{
    let field_id_name = id_name(ty.to_string());
    quote!{
        #name: match self.#name {
            Some(v) =>  #field_id_name::_set(&mut group.#name, v, 0),
            None => #field_id_name::None,
        }
    }
}

// 普通结构体或枚举取值 非必须的子组件插入容器， 如果没有， 返回Id::None(枚举组件)
fn struct_struct_field_builder(name: &syn::Ident, ty: &syn::Type) -> quote::__rt::TokenStream{
    quote!{
        #name: match self.#name {
            Some(v) => v,
            None => <#ty as std::default::Default>::default(),
        }
    }
}

// 构建枚举builder
fn enum_builder(name: &syn::Ident, variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> quote::__rt::TokenStream{
    let mut arr = Vec::new();
    arr.push(def_enum_builder(name));
    arr.push(impl_enum_builder_new(name));
    arr.push(impl_enum_build(name, variants));
    arr.push(impl_enum_variants_sets(name, variants));
    quote!{
        #(#arr)*
    }
}

fn def_enum_builder(name: &syn::Ident) -> quote::__rt::TokenStream{
    let builder_name = builder_name(name.to_string());
    quote!{
        pub struct #builder_name{
            value: Option<#name>
        }
    }
}

fn impl_enum_builder_new(name: &syn::Ident) -> quote::__rt::TokenStream{
    let builder_name = builder_name(name.to_string());
    quote!{
        impl #builder_name{
            pub fn new() -> #builder_name{
                #builder_name{
                    value: None,
                }
            }
        }
    }
}

fn impl_enum_build(name: &syn::Ident, variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> quote::__rt::TokenStream{
    let first_variant = match variants.first() {
        Some(v) => match v {
            syn::punctuated::Pair::Punctuated(v, _p) => v,
            syn::punctuated::Pair::End(v) => v,
        },
        None => panic!("enum variants len is 0"),
    };
    let first_field = match &first_variant.fields {
        syn::Fields::Unnamed(v) => match v.unnamed.first(){
            Some(v) => match v {
                syn::punctuated::Pair::End(v) => v,
                _ => panic!("enum variant's field len > 1"),
            },
            None => panic!("enum variant's field len is 0"),
        },
        _ => panic!("enum variant ty error"),
    };
    let first_field_ty_str = first_field.ty.clone().into_token_stream().to_string();
    let first_field_ty = &first_field.ty;
    let first_field_ty_builder_name = builder_name(first_field_ty_str.clone());
    let first_variant_name = &first_variant.ident;
    let first_variant_name_lowercase = ident(&first_variant.ident.to_string().to_lowercase());
    let builder_name = builder_name(name.to_string());
    let group_name = group_name(name.to_string());
    let builder_attrs = paser_builder_attrs(first_field);

    if builder_attrs.build_ty == BuildType::Builder {
        quote!{
            impl<C: ComponentMgr> Builder<C, #group_name<C>, #name> for #builder_name{
                fn build(self, group: &mut #group_name<C>) -> #name{
                    match self.value {
                        None => #name::#first_variant_name(Builder::build(#first_field_ty_builder_name::new(), &mut group.#first_variant_name_lowercase)),
                        Some(value) => value,
                    }
                }
            }
        }
    }else {
        quote!{
            impl<C: ComponentMgr> Builder<C, #group_name<C>, #name> for #builder_name{
                fn build(self, group: &mut #group_name<C>) -> #name{
                    match self.value {
                        None => #name::#first_variant_name(<#first_field_ty as std::default::Default>::default()),
                        Some(value) => value,
                    }
                }
            }
        }
    }
    
}

fn impl_enum_variants_sets(name: &syn::Ident, variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> quote::__rt::TokenStream{
    let mut arr = Vec::new();
    let builder_name = builder_name(name.to_string());
    for variant in variants.iter(){
        arr.push(enum_variant_set(name, variant));
    }

    quote!{
        impl #builder_name {
            #(#arr)*
        }
    }
}

// 枚举的每个变体的设置方法
fn enum_variant_set(name: &syn::Ident, variant: &syn::Variant) -> quote::__rt::TokenStream{
    let variant_name = &variant.ident;
    let variant_name_lowercase = ident(&variant_name.to_string().to_lowercase());
    let first_field = match &variant.fields {
        syn::Fields::Unnamed(v) => match v.unnamed.first(){
            Some(v) => match v {
                syn::punctuated::Pair::End(v) => v,
                _ => panic!("enum variant's field len > 1"),
            },
            None => panic!("enum variant's field len is 0"),
        },
        _ => panic!("enum variant ty error"),
    };
    let field_ty = &first_field.ty;
    quote!{
        pub fn #variant_name_lowercase(mut self, value: #field_ty) -> Self{
            self.value = Some(#name::#variant_name(value));
            self
        }
    }
}

fn builder_name(s: String) -> syn::Ident{
    ident(&(s + "Builder"))
}

fn paser_builder_attrs(field: &syn::Field) -> BuilderAttrs{
    let attrs = &field.attrs;
    let mut b_attrs = BuilderAttrs{
        is_export: false,
        build_ty: BuildType::None,
    };
    for a in attrs.iter(){
        if a.path.clone().into_token_stream().to_string().as_str() == "Builder" {
            let meta = a.parse_meta().unwrap();
            match meta {
                syn::Meta::List(list) => {
                    for nested in list.nested.iter() {
                        match nested {
                            syn::NestedMeta::Meta(meta) => {
                                match meta {
                                    syn::Meta::List(list) => {
                                        if list.ident.to_string() == "Build" {
                                            for nested in list.nested.iter() {
                                                match nested {
                                                    syn::NestedMeta::Meta(meta) => {
                                                        match meta {
                                                            syn::Meta::Word(word) => {
                                                                if word.to_string() == "Builder" {
                                                                    b_attrs.build_ty = BuildType::Builder;
                                                                }else if word.to_string() == "Default" {
                                                                    b_attrs.build_ty = BuildType::Default;
                                                                }else {
                                                                    panic!("error attr : {}", word.to_string());
                                                                }
                                                            },
                                                            _ => panic!("Builder inner attr is not meta"),
                                                        }
                                                    }
                                                    _ => panic!("Builder inner attr is not meta")
                                                }
                                            }
                                        } else {
                                            panic!("error attr : {}", list.ident.to_string());
                                        }
                                    },
                                    syn::Meta::Word(word) => {
                                        if word.to_string() == "Export" {
                                            b_attrs.is_export = true;
                                        }else {
                                            panic!("error attr : {}", word.to_string());
                                        }
                                    },
                                    _ => panic!("Builder inner attr is not list or world"),
                                }
                            },
                            _ => panic!("Builder inner attr is not meta")
                        }
                    }
                },
                _ => panic!("Builder attr is not list"),
            }
        }
    }
    b_attrs
}

struct BuilderAttrs{
    is_export: bool,
    build_ty: BuildType,
}

#[derive(PartialEq, Eq)]
enum BuildType{
    Builder,
    Default,
    None
}

