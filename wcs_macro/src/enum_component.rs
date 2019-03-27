use crate::proc_macro::TokenStream;
use quote::{quote, ToTokens};

use data::*;

pub fn impl_enum_component_macro(enum_data: &EnumData) -> TokenStream {
    let p = id_unnamed(&enum_data);
    let r = ref_unnamed(&enum_data);
    let g = group_unnamed(&enum_data);
    let c_d = impl_create_destroy(&enum_data);

    let gen = quote!{
        #p
        #r
        #g
        #c_d
    };
    gen.into()
}

fn id_unnamed(enum_data: &EnumData) -> quote::__rt::TokenStream {
    let EnumData{name:_, component_data, variants} = enum_data;
    let ComponentData {group_name:_, id_name, write_ref_name:_, read_ref_name:_, is_must:_, c_type:_} = component_data;
    let mut id_impls = Vec::new();
    for variant in variants.data.iter(){
        let Variant{key, fields:_} = variant;
        id_impls.push(quote!{
            #key(usize)
        });
    }
    let variant = &variants.data[0];
    let Variant{key, fields:_} = variant;
    quote!{
        #[derive(Clone, Debug)]
        pub enum #id_name{
            None,
            #(#id_impls),*
        }

        impl Default for #id_name{
            fn default() -> #id_name {
                #id_name::None
            }
        }
    }
}

fn c_data(field: &Field) -> ComponentData {
    let ty = field.ty.clone().into_token_stream().to_string();
    ComponentData{
        group_name: group_name(ty.clone()),
        id_name: id_name(ty.clone()),
        write_ref_name: write_ref_name(ty.clone()),
        read_ref_name: read_ref_name(ty.clone()),
        is_must: false,
        c_type: ident("xx"),
    }
}

fn ref_unnamed(enum_data: &EnumData) -> quote::__rt::TokenStream {
    let EnumData{name:_, component_data, variants} = enum_data;
    let ComponentData {group_name, id_name, write_ref_name, read_ref_name, is_must:_, c_type:_} = component_data;
    let mut read_ref_variant = Vec::new();
    let mut write_ref_variant = Vec::new();
    let mut feild_write_new = Vec::new();
    let mut feild_read_new = Vec::new();
    let read_ref_name1 = &read_ref_name;
    let write_ref_name1 = &write_ref_name;
    for variant in variants.data.iter(){
        let Variant{key, fields} = variant;
        let field = &fields.data[0];
        let ComponentData {group_name:_, id_name: _, write_ref_name, read_ref_name, is_must:_, c_type:_} = c_data(&field);
        
        let name = ident(&key.to_string().to_lowercase());
        read_ref_variant.push( quote!{
            #key(#read_ref_name<'a, M>)
        });
        write_ref_variant.push( quote!{
            #key(#write_ref_name<'a, M>)
        });
        feild_read_new.push(quote!{
            #id_name::#key(id) => #read_ref_name1::#key(#read_ref_name::new(id, &g.#name))
        });
        feild_write_new.push(quote!{
            #id_name::#key(id) => #write_ref_name1::#key(#write_ref_name::new(id, g.#name.to_usize(), m))
        });
    }

    quote!{

        pub enum #read_ref_name<'a, M: ComponentMgr>{
            None,
            #(#read_ref_variant),*
        }

        impl<'a, M: ComponentMgr> #read_ref_name<'a, M>{
            pub fn new(p: #id_name, g: &#group_name<M>) -> #read_ref_name<M>{
                match p {
                    #id_name::None => #read_ref_name::None,
                    #(#feild_read_new),*
                }
            }
        }

        pub enum #write_ref_name<'a, M: ComponentMgr>{
            None,
            #(#write_ref_variant),*
        }

        impl<'a, M: ComponentMgr> #write_ref_name<'a, M>{
            pub fn new(p: #id_name, g: usize, m: &mut M) -> #write_ref_name<M>{
                let g = #group_name::<M>::from_usize(g);
                match p {
                    #id_name::None => #write_ref_name::None,
                    #(#feild_write_new),*
                }
            }
        }
    }
}

fn group_unnamed(enum_data: &EnumData) -> quote::__rt::TokenStream {
    let EnumData{name: _, component_data, variants} = enum_data;
    let ComponentData {group_name, id_name:_, write_ref_name: _, read_ref_name:_, is_must:_, c_type:_} = component_data;
    let mut arr_group = Vec::new();
    let mut arr_name = Vec::new();
    for variant in variants.data.iter(){
        let Variant{key, fields} = variant;
        let field = &fields.data[0];
        let ComponentData {group_name, id_name: _, write_ref_name: _, read_ref_name: _, is_must:_, c_type:_} = c_data(&field);
        let name = ident(&key.to_string().to_lowercase());
        arr_name.push(name);
        arr_group.push(group_name);
    }

    let arr_group1 = arr_group.clone();
    let arr_name1 = arr_name.clone();
    quote!{
        pub struct #group_name<M: ComponentMgr>{
            #(pub #arr_name: #arr_group<M>),*
        }

        impl<M: ComponentMgr> ComponentGroupTree for #group_name<M>{
            // type C = M;
            // fn new () -> #group_name<M>{
            //     #group_name{#(#arr_name1: #arr_group1::<M>::new()),*}
            // }
        }

        impl<M: ComponentMgr> std::default::Default for #group_name<M>{
            fn default() -> Self {
                #group_name{#(#arr_name1: #arr_group1::<M>::default()),*}
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
    }
}

fn variant_insert(variant: &Variant, name: &syn::Ident, id_name: &syn::Ident) -> quote::__rt::TokenStream {
    let Variant{key, fields} = variant;
    let field = &fields.data[0];
    let ComponentData {group_name:_, id_name: _, write_ref_name: _, read_ref_name: _, is_must:_, c_type:_} = c_data(&field);
    let f_name = ident(&key.to_string().to_lowercase());
    quote!{
        #name::#key(value) => #id_name::#key(groups.#f_name._group.insert(value, parent))
    }
}

fn variant_set_notify(variant: &Variant, pre: &syn::Ident) -> quote::__rt::TokenStream {
    let Variant{key, fields} = variant;
    let field = &fields.data[0];
    let ComponentData {group_name: _, id_name: _, write_ref_name: _, read_ref_name: _, is_must:_, c_type:_} = c_data(&field);

    quote!{
        #pre::#key(w_ref) => {
            w_ref.create_notify();
            // let groups = #group_name::<M>::from_usize(w_ref.groups);
            // let handlers = groups._group.get_handlers();
            // let parent = groups._group.get(w_ref.id).parent;
            // notify(Event::Create{id: w_ref.id, parent: parent}, &handlers.borrow(), &mut w_ref.mgr);
        }
    }
}

fn variant_recursive_destroy(variant: &Variant, pre: &syn::Ident,) -> quote::__rt::TokenStream {
    let Variant{key, fields} = variant;
    let field = &fields.data[0];
    let ComponentData {group_name:_, id_name: _, write_ref_name: _, read_ref_name: _, is_must:_, c_type:_} = c_data(&field);
    quote!{
        #pre::#key(w_ref) => w_ref.destroy()
    }
}

fn variant_recursive_setparent(variant: &Variant, pre: &syn::Ident,) -> quote::__rt::TokenStream {
    let Variant{key, fields} = variant;
    let field = &fields.data[0];
    let ComponentData {group_name:_, id_name: _, write_ref_name: _, read_ref_name: _, is_must:_, c_type:_} = c_data(&field);
    quote!{
        #pre::#key(w_ref) => w_ref.set_parent(parent)
    }
}


pub fn impl_create_destroy(enum_data: &EnumData) -> quote::__rt::TokenStream {
    let EnumData{name, component_data, variants} = enum_data;
    let ComponentData {group_name, id_name, write_ref_name, read_ref_name:_, is_must:_, c_type:_} = component_data;
    let mut inserts = Vec::new();
    let mut arr_name = Vec::new();
    let mut notifys = Vec::new();
    let mut destroys = Vec::new();
    let mut set_parents = Vec::new();
    for variant in variants.data.iter(){
        inserts.push(variant_insert(variant, name, id_name));
        arr_name.push(name);
        notifys.push(variant_set_notify(variant, write_ref_name));
        destroys.push(variant_recursive_destroy(variant, write_ref_name));
        set_parents.push(variant_recursive_setparent(variant, write_ref_name))
    }

    quote! {
        impl #id_name{
            pub fn _set<M: ComponentMgr>(groups: &mut #group_name<M>, v: #name, parent: usize) -> #id_name{
                match v {
                    #(#inserts),*
                }
            }
        }

        impl<'a, M: ComponentMgr> #write_ref_name<'a, M>{
            // pub fn _set(v: #name, parent: usize, group: usize, mgr: &'a mut M) -> #write_ref_name<'a, M>{
            //     let groups = #group_name::<M>::from_usize_mut(group);
            //     let id = #id_name::_set(groups, v, parent);
            //     let mut r = #write_ref_name::new(id, group, mgr);
            //     r.create_notify();
            //     r
            // }

            pub fn create_notify(&mut self){
                match self {
                    #write_ref_name::None => (),
                    #(#notifys),*
                }
            }

            //递归设置parent
            pub fn set_parent(&mut self, parent: usize){
                match self {
                    #write_ref_name::None => (),
                    #(#set_parents),*
                }
                // let groups = #g_name::<M>::from_usize(self.groups);
                // {
                //     let value = groups._group.get(self.id);
                //     value.parent = parent;
                //     let parent = self.id;
                //     #(#field_set_parent)*
                // }
            }

            pub fn destroy(&mut self){
                match self {
                    #write_ref_name::None => (),
                    #(#destroys),*
                }
            }
        }
    }
}
