use crate::proc_macro::TokenStream;
use quote::quote;

use data::*;

pub fn impl_enum_component_macro(enum_data: &EnumData) -> TokenStream {
    let p = def_point_named(&enum_data);
    let p_d = impl_point_default_named(&enum_data);
    let r = def_ref_named(&enum_data);
    let g = def_impl_group(&enum_data);
    let c_d = impl_create_destroy(&enum_data);

    let gen = quote!{
        #p
        #p_d
        #r
        #g
        #c_d
    };
    gen.into()
}

fn def_point_named(enum_data: &EnumData) -> quote::__rt::TokenStream {
    let EnumData{name:_, component_data, variants} = enum_data;
    let ComponentData {group_name:_, point_name, write_ref_name:_, read_ref_name:_, is_must:_, c_type:_} = component_data;
    let mut point_impls = Vec::new();
    for variant in variants.data.iter(){
        let Variant{key, fields} = variant;
        let mut arr_point = &fields.point_names;
        let mut arr_name = &fields.keys;
        point_impls.push(quote!{
            #key{
                #(#arr_name: #arr_point),*
            }
        });
    }
    quote!{
        #[derive(Clone, Debug)]
        pub enum #point_name{
            #(#point_impls),*
        }

        impl Point for #point_name {}
    }
}

fn impl_point_default_named(enum_data: &EnumData) -> quote::__rt::TokenStream {
    let EnumData{name:_, component_data, variants} = enum_data;
    let ComponentData {group_name:_, point_name, write_ref_name:_, read_ref_name:_, is_must:_, c_type:_} = component_data;
    let variant = &variants.data[0];
    let Variant{key, fields} = variant;
    let arr_point = &fields.point_names;
    let arr_name = &fields.keys;

    quote!{
        impl Default for #point_name{
            fn default() -> #point_name {
                #point_name::#key{
                    #(#arr_name: #arr_point(0)),*
                }
            }
        }
    }
}

fn def_ref_named(enum_data: &EnumData) -> quote::__rt::TokenStream {
    let EnumData{name:_, component_data, variants} = enum_data;
    let ComponentData {group_name, point_name, write_ref_name, read_ref_name, is_must:_, c_type:_} = component_data;
    let mut read_ref_variant = Vec::new();
    let mut write_ref_variant = Vec::new();
    let mut feild_write_new = Vec::new();
    let mut feild_read_new = Vec::new();
    for variant in variants.data.iter(){
        let Variant{key, fields} = variant;
        let mut arr_write_ref = &fields.write_ref_names;
        let mut arr_read_ref = &fields.read_ref_names;
        let mut arr_write_ref1 = &fields.write_ref_names;
        let mut arr_read_ref1 = &fields.read_ref_names;
        let mut arr_name = &fields.keys;
        let mut arr_name1 = &fields.keys;
        let mut arr_name2 = &fields.keys;
        let mut arr_name3 = &fields.keys;
        let mut arr_name5 = &fields.keys;
        let mut arr_name6 = &fields.keys;
        let mut arr_name7 = &fields.keys;
        let mut arr_name8 = &fields.keys;
        let mut group_names = Vec::new();
        for name in fields.keys.iter(){
            group_names.push(ident(&format!("{}_{}", key.to_string().to_lowercase(), name)));
        }
        let mut group_names1 = group_names.clone();
        read_ref_variant.push( quote!{
            #key{
                #(#arr_name: #arr_read_ref<'a, M>),*
            }
        });
        write_ref_variant.push( quote!{
            #key{
                #(#arr_name1: #arr_write_ref<'a, M>),*
            }
        });
        feild_read_new.push(quote!{
            #point_name::#key{#(#arr_name5),*} => {
                #read_ref_name::#key{
                    #(#arr_name2: #arr_read_ref1::new(#arr_name7, &g.#group_names)),*
                }
            }
        });
        feild_write_new.push(quote!{
            #point_name::#key{#(#arr_name6),*} => {
                #write_ref_name::#key{
                    #(#arr_name3: #arr_write_ref1::new(#arr_name8, g.#group_names1.to_usize(), m)),*
                }
            }
        });
    }

    quote!{
        pub enum #write_ref_name<'a, M: ComponentMgr>{
            #(#write_ref_variant),*
        }

        pub enum #read_ref_name<'a, M: ComponentMgr>{
            #(#read_ref_variant),*
        }

        impl<'a, M: ComponentMgr> #write_ref_name<'a, M>{
            pub fn new(p: #point_name, g: usize, m: &mut M) -> #write_ref_name<M>{
                let g = #group_name::<M>::from_usize(g);
                match p {
                    #(#feild_write_new),*
                }
            }
        }

        impl<'a, M: ComponentMgr> #read_ref_name<'a, M>{
            pub fn new(p: #point_name, g: &#group_name<M>) -> #read_ref_name<M>{
                match p {
                    #(#feild_read_new),*
                }
            }
        }
    }
}

fn def_impl_group(enum_data: &EnumData) -> quote::__rt::TokenStream {
    let EnumData{name: _, component_data, variants} = enum_data;
    let ComponentData {group_name, point_name:_, write_ref_name: _, read_ref_name:_, is_must:_, c_type:_} = component_data;
    let mut arr_group = Vec::new();
    let mut arr_name = Vec::new();
    for variant in variants.data.iter(){
        let Variant{key, fields} = variant;
        for group_name in fields.group_names.iter(){
            arr_group.push(group_name);
        }
        for name in fields.keys.iter(){
            arr_name.push(ident(&format!("{}_{}", key.to_string().to_lowercase(), name)));
        }
    }

    let arr_group1 = arr_group.clone();
    let arr_name1 = arr_name.clone();
    quote!{
        pub struct #group_name<M: ComponentMgr>{
            #(pub #arr_name: #arr_group<M>),*
        }

        impl<M: ComponentMgr> ComponentGroupTree for #group_name<M>{
            type C = M;
            fn new () -> #group_name<M>{
                #group_name{#(#arr_name1: #arr_group1::<M>::new()),*}
            }
        }

        impl<M: ComponentMgr> #group_name<M>{
            #[inline]
            pub fn to_usize (&self) -> usize{
                self as *const #group_name<M> as usize
            }

            #[inline]
            pub fn from_usize<'a> (ptr: usize) -> &'a #group_name<M>{
                unsafe{&*(ptr as *const #group_name<M>)}
            }

            #[inline]
            pub fn from_usize_mut<'a>(ptr: usize) -> &'a mut #group_name<M>{
                unsafe{&mut *(ptr as *mut #group_name<M>)}
            }
        }

        // impl<M: ComponentMgr> #group_name<M>{
        //     pub fn insert(&mut self, component: #name, parent: usize) -> #point_name{
        //         // match component {
        //         //     #(#arr_name_enum::#arr_key() =>{

        //         //     }),*
        //         // }
        //         // let index = self.components.insert(ComponentP::new(component, parent));
        //         // let mut point = P::default();
        //         // point.set_id(index);
        //         // point
        //     }

        //     pub fn remove(&mut self, component: #point_name){
        //         // match component {
        //         //     #(#arr_name_enum::#arr_key() =>{

        //         //     }),*
        //         // }
        //         // let index = self.components.insert(ComponentP::new(component, parent));
        //         // let mut point = P::default();
        //         // point.set_id(index);
        //         // point
        //     }
        // }
    }
}

fn variant_insert(variant: &Variant, name: &syn::Ident, point_name: &syn::Ident) -> quote::__rt::TokenStream {
    let Variant{key, fields} = variant;
    let field_names = &fields.keys;
    let field_names1 = &fields.keys;
    let field_names2 = &fields.keys;
    let mut group_names = Vec::new();
    for name in fields.keys.iter(){
        group_names.push(ident(&format!("{}_{}", key.to_string().to_lowercase(), name)));
    }
    match fields.ty {
        FieldsType::Named => {
            quote!{
                #name::#key{#(#field_names),*} => {
                    #point_name::#key{
                        #(#field_names2: groups.#group_names._group.insert(#field_names1, parent.clone())),*
                    }
                }
            }
        },
        FieldsType::Unnamed  => {
            quote!{
                #name::#key(#(#field_names),*) => {
                    #point_name::#key{#(#field_names2: groups.#group_names._group.insert(#field_names1, parent.clone())),*}
                }
            }
        }
    }
}

fn variant_set_notify(variant: &Variant, pre: &syn::Ident) -> quote::__rt::TokenStream {
    let Variant{key, fields} = variant;
    let field_names = &fields.keys;
    let field_names1 = &fields.keys;
    let field_names2 = &fields.keys;
    let mut group_names = Vec::new();
    let mut group_tys = Vec::new();
    for name in fields.keys.iter(){
        group_names.push(ident(&format!("{}_{}", key.to_string().to_lowercase(), name)));
    }
    for group_ty in fields.group_names.iter(){
        group_tys.push(group_ty.clone());
    }

    quote!{
        #pre::#key{#(#field_names),*} => {
        #(
            let groups = #group_tys::<M>::from_usize(#field_names.groups);
            let handlers = groups._group.get_handlers();
            notify(Event::Create{point: #field_names1.point.clone(), parent: parent.clone()}, &handlers.borrow(), &mut #field_names2.mgr);
        )*
        }
    }
}

fn variant_recursive_destroy(variant: &Variant, pre: &syn::Ident,) -> quote::__rt::TokenStream {
    let Variant{key, fields} = variant;
    let field_names = &fields.keys;
    let field_names1 = &fields.keys;
    let field_names2 = &fields.keys;
    let field_names3 = &fields.keys;
    let write_ref_names = &fields.write_ref_names;
    let mut group_names = Vec::new();
    for name in fields.keys.iter(){
        group_names.push(ident(&format!("{}_{}", key.to_string().to_lowercase(), name)));
    }
    quote!{
        #pre::#key{#(#field_names),*} => {
        #(
            #write_ref_names::new(#field_names1.point.clone(), #field_names2.groups, #field_names3.mgr).destroy();
        )*
        }
    }
}


pub fn impl_create_destroy(enum_data: &EnumData) -> quote::__rt::TokenStream {
    let EnumData{name, component_data, variants} = enum_data;
    let ComponentData {group_name, point_name, write_ref_name, read_ref_name:_, is_must:_, c_type:_} = component_data;
    let mut inserts = Vec::new();
    let mut arr_name = Vec::new();
    let mut notifys = Vec::new();
    let mut destroys = Vec::new();
    for variant in variants.data.iter(){
        inserts.push(variant_insert(variant, name, point_name));
        arr_name.push(name);
        notifys.push(variant_set_notify(variant, write_ref_name));
        destroys.push(variant_recursive_destroy(variant, write_ref_name));
    }

    quote! {
        impl #point_name{
            fn _set<M: ComponentMgr>(groups: &mut #group_name<M>, v: #name, parent: &usize) -> #point_name{
                match v {
                    #(#inserts),*
                }
            }
        }

        impl<'a, M: ComponentMgr> #write_ref_name<'a, M>{
            pub fn _set(v: #name, parent: &usize, group: usize, mgr: &'a mut M) -> #write_ref_name<'a, M>{
                let groups = #group_name::<M>::from_usize_mut(group);
                let point = #point_name::_set(groups, v, parent);
                let mut r = #write_ref_name::new(point, group, mgr);
                r._set_notify(parent);
                r
            }

            pub fn _set_notify(&mut self, parent: &usize){
                match self {
                    #(#notifys)*
                }
            }

            pub fn destroy(&mut self){
                match self {
                    #(#destroys)*
                }
            }
        }
    }
}
