use proc_macro::TokenStream;
use quote::quote;

use data::*;

pub fn impl_component_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = match &ast.data {
        syn::Data::Struct(s) => {
            impl_struct(name, s)
        },
        syn::Data::Enum(_) => {
            //impl_enum()
            panic!("xxxx")
        },
        syn::Data::Union(_) => panic!("xxxx"),
    };
    gen.into()
}

pub fn impl_struct(name: &syn::Ident, s: &syn::DataStruct) -> quote::__rt::TokenStream {
    match &s.fields {
        syn::Fields::Named(f) => {
            let fields = Fields::from(&f.named, FieldsType::Named);
            let mut arr = Vec::new();
            // let fields = &f.named;
            arr.push(def_point(name));
            arr.push(impl_struct_point(name, &fields));
            arr.push(component_group_tree(name, &fields));
            arr.push(component_impl_create(name, &fields));
            quote! {
                #(#arr)*
            }
        },
        syn::Fields::Unnamed(f) => {
            let fields = Fields::from(&f.unnamed, FieldsType::Named);
            let mut arr = Vec::new();
            // let fields = &f.named;
            arr.push(def_point(name));
            arr.push(impl_struct_point(name, &fields));
            arr.push(component_group_tree(name, &fields));
            arr.push(component_impl_create(name, &fields));
            quote! {
                #(#arr)*
            }
        },
        syn::Fields::Unit => panic!("xxxx")
    }
}

// fn impl_enum(name: &syn::Ident, s: &syn::DataEnum) -> quote::__rt::TokenStream {
//     let variants = &s.variants;
//     for v in variants.iter(){

//     }
//     match &s.variants {
//         syn::Fields::Named(f) => {
//             let mut arr = Vec::new();
//             let fields = &f.named;
//             arr.push(def_point(name));
//             arr.push(impl_struct_point(name, fields));
//             arr.push(component_group_tree(name, fields));
//             quote! {
//                 #(#arr)*
//             }
//         },
//         syn::Fields::Unnamed(f) => panic!("xxxx"),
//         syn::Fields::Unit => panic!("xxxx")
//     }
// }

pub fn def_point(name: &syn::Ident) -> quote::__rt::TokenStream {
    let point = point_name(name.to_string());
    let read_ref = read_ref_name(name.to_string());
    let write_ref = write_ref_name(name.to_string());
    let group = group_name(name.to_string());
    quote! {
        #[derive(Clone, Default, Debug)]
        pub struct #point(pub usize);
        
        pub struct #read_ref<'a, M: ComponentMgr>{
            pub point: #point,
            groups: &'a #group<M>,
        }

        pub struct #write_ref<'a, M: ComponentMgr>{
            pub point: #point,
            groups: usize,
            mgr: &'a mut M,
        }
    }
}

pub fn impl_point(name: &syn::Ident, point_impls: &Vec<quote::__rt::TokenStream>, readref_impls: &Vec<quote::__rt::TokenStream>, writeref_impls: &Vec<quote::__rt::TokenStream>) -> quote::__rt::TokenStream {
    let point = point_name(name.to_string());
    let group = group_name(name.to_string());
    let read_reff = read_ref_name(name.to_string());
    let write_reff = write_ref_name(name.to_string());
    quote! {
        impl ID for #point{
            fn id(& self) -> usize{
                self.0
            }
            fn set_id(&mut self, id: usize){
                self.0 = id;
            }
        }

        impl Point for #point{}

        impl Deref for #point{
            type Target = usize;
            fn deref(&self) -> &usize{
                &self.0
            }
        }

        impl DerefMut for #point{
            fn deref_mut(&mut self) -> &mut usize{
                &mut self.0
            }
        }

        impl #point{
            #(#point_impls)*
        }

        impl<'a, M: ComponentMgr> #read_reff<'a, M>{
            #(#readref_impls)*

            pub fn new(p: #point, g: &#group<M>) -> #read_reff<M>{
                #read_reff{
                    point: p,
                    groups: g,
                }
            }
        }

        impl<'a, M: ComponentMgr> Deref for #read_reff<'a, M>{
            type Target = #point;
            fn deref(&self) -> &#point{
                &self.point
            }
        }

        impl<'a, M: ComponentMgr> #write_reff<'a, M>{
            #(#writeref_impls)*

            pub fn modify<F: FnOnce(&mut #name) -> bool>(&mut self, m: F) {
                let groups = #group::<M>::from_usize_mut(self.groups);
                let handlers = groups._group.get_handlers();
                let mut elem = groups._group.get_mut(&self.point);
                if m(&mut elem) {
                    notify(Event::ModifyField{
                        point: self.point.clone(),
                        parent: elem.parent,
                        field: ""
                    }, &handlers.borrow(), &mut self.mgr);
                }
            }

            pub fn new(p: #point, g: usize, m: &mut M) -> #write_reff< M>{
                #write_reff{
                    point: p,
                    groups: g,
                    mgr: m,
                }
            }
        }

        impl<'a, M: ComponentMgr> Deref for #write_reff<'a, M>{
            type Target = #point;
            fn deref(&self) -> &#point{
                &self.point
            }
        }
        impl<'a, M: ComponentMgr> DerefMut for #write_reff<'a, M>{
            fn deref_mut(&mut self) -> &mut #point{
                &mut self.point
            }
        }
    }
}

pub fn impl_struct_point(name: &syn::Ident, fields: &Fields) -> quote::__rt::TokenStream {
    let mut point_impls = Vec::new();
    let mut readref_impls = Vec::new();
    let mut writeref_impls = Vec::new();
    
    for f in fields.data.iter(){
        point_impls.push(impl_struct_point_fun(name, f));
        readref_impls.push(impl_struct_readref_fun( f));
        writeref_impls.push(impl_struct_writeref_fun(name, f));
    }

    impl_point(name, &point_impls, &readref_impls, &writeref_impls)
}

pub fn impl_struct_point_fun(name: &syn::Ident, field: &Field) -> quote::__rt::TokenStream {
    let group = group_name(name.to_string());
    let Field{key, ty, set_name, get_name, get_mut_name, ty_name:_, mark, key_str:_} = field;
    match mark {
        FieldMark::Component(data) => {
            let ComponentData {group_name:_, point_name, write_ref_name:_, read_ref_name:_, is_must:_, c_type} = data;
            quote! {
                pub fn #set_name<M: ComponentMgr>(&self, value: #c_type, groups: &mut #group<M>) -> usize{
                    let index = groups.#key._group.insert(value, self.0.clone());
                    let elem = groups._group.get_mut(self);
                    elem.owner.#key = index;
                    elem.parent
                }

                pub fn #get_name<M: ComponentMgr>(&self, groups: &#group<M>) -> #point_name{
                    groups._group.get(self).#key.clone()
                }
            }
        },
        FieldMark::EnumComponent(data) => {
            let ComponentData {group_name:_, point_name, write_ref_name:_, read_ref_name:_, is_must:_, c_type} = data;
            quote! {
                pub fn #set_name<M: ComponentMgr>(&self, value: #c_type, groups: &mut #group<M>) -> usize{
                    let index = #point_name::_set(&mut groups.#key, value, &self.0);
                    let elem = groups._group.get_mut(self);
                    elem.owner.#key = index;
                    elem.parent
                }

                pub fn #get_name<M: ComponentMgr>(&self, groups: &#group<M>) -> #point_name{
                    groups._group.get(self).#key.clone()
                }
            }
        },
        FieldMark::Data => {
            quote! {
                pub fn #set_name<M: ComponentMgr>(&self, value: #ty, groups: &mut #group<M>) -> usize{
                    let elem = groups._group.get_mut(self);
                    elem.owner.#key = value;
                    elem.parent
                }

                pub fn #get_name<'a, M: ComponentMgr>(&self, groups: &'a #group<M>) -> &'a #ty{
                    &(groups._group.get(self).#key)
                }

                pub fn #get_mut_name<'a, M: ComponentMgr>(&self, groups: &'a mut #group<M>) -> &'a mut #ty{
                    &mut (groups._group.get_mut(self).#key)
                }
            }
        }
    }
}

pub fn impl_struct_readref_fun(field: &Field) -> quote::__rt::TokenStream {
    let Field{key, ty, set_name:_, get_name, get_mut_name:_, ty_name:_, mark, key_str:_} = field;
    match mark {
        FieldMark::Component(data) => {
            let ComponentData {group_name:_, point_name:_, write_ref_name:_, read_ref_name, is_must:_,c_type:_} = data;
            quote! {
            pub fn #get_name(&self) -> #read_ref_name<M>{
                    let p = self.point.#get_name(self.groups).clone();
                    #read_ref_name::new(p, &self.groups.#key)
                }
            }
        },
        FieldMark::EnumComponent(data) => {
            let ComponentData {group_name:_, point_name:_, write_ref_name:_, read_ref_name, is_must:_, c_type:_} = data;
            quote! {
                pub fn #get_name(&self) -> #read_ref_name<M>{
                    let p = self.point.#get_name(self.groups).clone();
                    #read_ref_name::new(p, &self.groups.#key)
                }
            }
        },
        FieldMark::Data => {
            quote! {
                pub fn #get_name(&self) -> &#ty{
                    unsafe{&*(self.point.#get_name(self.groups) as *const #ty)}
                }
            }
        }
    }
}

pub fn impl_struct_writeref_fun(name: &syn::Ident, field: &Field) -> quote::__rt::TokenStream {
    let group = group_name(name.to_string());
    let Field{key, ty, set_name, get_name, get_mut_name, ty_name:_, mark, key_str} = field;
    match mark {
        FieldMark::Component(data) => {
            let ComponentData {group_name:_, point_name:_, write_ref_name, read_ref_name, is_must:_, c_type} = data;
            quote! {
                pub fn #set_name(&mut self, value: #c_type){
                    let groups = #group::<M>::from_usize_mut(self.groups);
                    {
                        let old = self.point.#get_name(groups).clone();
                        let mut old_ref = #write_ref_name::<M>::new(old, groups.#key.to_usize(), &mut self.mgr);
                        old_ref.destroy(); //销毁
                    }
                    let parent = self.point.#set_name(value, groups);
                    let handlers = groups._group.get_handlers();
                    let handlers1 = groups.#key._group.get_handlers();
                    let new_point = self.point.#get_name(groups).clone();
                    //创建事件
                    notify(Event::Create{
                        point: new_point,
                        parent: parent,
                    }, &handlers1.borrow(), &mut self.mgr);

                    //修改事件
                    notify(Event::ModifyField{
                        point: self.point.clone(),
                        parent: parent,
                        field: #key_str
                    }, &handlers.borrow(), &mut self.mgr);
                }

                pub fn #get_name(&self) -> #read_ref_name<M>{
                    let groups = #group::<M>::from_usize(self.groups);
                    let p = self.point.#get_name(groups).clone();
                    #read_ref_name::new(p, &groups.#key)
                }

                pub fn #get_mut_name(&mut self) -> #write_ref_name<M>{
                    let groups = #group::<M>::from_usize_mut(self.groups);
                    let p = self.point.#get_name(groups).clone();
                    #write_ref_name::new(p, groups.#key.to_usize(), &mut self.mgr)
                }
            }
        },
        FieldMark::EnumComponent(data) => {
            let ComponentData {group_name:_, point_name:_, write_ref_name, read_ref_name, is_must:_, c_type} = data;
            quote! {
                pub fn #set_name(&mut self, value: #c_type){
                    let groups = #group::<M>::from_usize_mut(self.groups);
                    {
                        let old = self.point.#get_name(groups).clone();
                        let mut old_ref = #write_ref_name::<M>::new(old, groups.#key.to_usize(), &mut self.mgr);
                        old_ref.destroy(); //销毁
                    }
                    let parent = self.point.#set_name(value, groups);
                    {
                        let new_point = self.point.#get_name(groups).clone();
                        let mut new_write = #write_ref_name::<M>::new(new_point, groups.#key.to_usize(), &mut self.mgr);
                        new_write._set_notify(&parent);
                    }

                    let handlers = groups._group.get_handlers();
                    //修改事件
                    notify(Event::ModifyField{
                        point: self.point.clone(),
                        parent: parent,
                        field: #key_str
                    }, &handlers.borrow(), &mut self.mgr);
                }

                pub fn #get_name(&self) -> #read_ref_name<M>{
                    let groups = #group::<M>::from_usize(self.groups);
                    let p = self.point.#get_name(groups).clone();
                    #read_ref_name::new(p, &groups.#key)
                }

                pub fn #get_mut_name(&mut self) -> #write_ref_name<M>{
                    let groups = #group::<M>::from_usize_mut(self.groups);
                    let p = self.point.#get_name(groups).clone();
                    #write_ref_name::new(p, groups.#key.to_usize(), &mut self.mgr)
                }
            }
        },
        FieldMark::Data => {
            quote! {
                pub fn #set_name(&mut self, value: #ty){
                    let groups = #group::<M>::from_usize_mut(self.groups);
                    let parent = self.point.#set_name(value, groups);
                    let handlers = groups._group.get_handlers();
                    notify(Event::ModifyField{
                        point: self.point.clone(),
                        parent: parent,
                        field: #key_str
                    }, &handlers.borrow(), &mut self.mgr);
                }

                pub fn #get_name(&self) -> &#ty{
                    let groups = #group::<M>::from_usize(self.groups);
                    unsafe{&*(self.point.#get_name(groups) as *const #ty)}
                }

                pub fn #get_mut_name(&self) -> &mut #ty{
                    let groups = #group::<M>::from_usize_mut(self.groups);
                    unsafe{&mut *(self.point.#get_mut_name(groups) as *mut #ty)}
                }
            }
        }
    }
}

pub fn component_group_tree(name: &syn::Ident, fields: &Fields) -> quote::__rt::TokenStream {
    let mut field_types = Vec::new();
    let mut field_news = Vec::new();
    let Fields {ty:_, data} = fields;
    // let mut set_mgrs = Vec::new();
    for field in data.iter(){
        let Field{key, ty:_, set_name:_, get_name:_, get_mut_name:_, ty_name:_, mark, key_str:_} = field;
        let ComponentData {group_name, point_name:_, write_ref_name:_, read_ref_name:_, is_must:_, c_type:_} = match mark {
            FieldMark::Component(data)  => data,
            FieldMark::EnumComponent(data)  => data,
            _ => continue,
        };
        field_types.push(quote! {
            pub #key: #group_name<M>,
        });
        field_news.push(quote! {
            #key: #group_name::new(),
        });
    }

    let group_name = group_name(name.to_string());
    let point_name = point_name(name.to_string());

    quote! {
        pub struct #group_name<M: ComponentMgr>{
            pub _group: ComponentGroup<#name, #point_name, M>,
            #(#field_types)*
        }

        impl<M: ComponentMgr> ComponentGroupTree for #group_name<M>{
            type C = M;
            fn new () -> #group_name<M>{
                #group_name{
                    #(#field_news)*
                    _group: ComponentGroup::new(),
                }
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

pub fn component_impl_create(name: &syn::Ident, fields: &Fields) -> quote::__rt::TokenStream {
    let mut field_creates = Vec::new();
    let mut field_create_notifys = Vec::new();
    let mut field_destroys = Vec::new();
    let p_name = point_name(name.to_string());
    let w_r_name = write_ref_name(name.to_string());
    let g_name = group_name(name.to_string());
    let Fields {ty:_, data} = fields;
    for field in data.iter(){
        let Field{key, ty, set_name:_, get_name:_, get_mut_name:_, ty_name: _, mark, key_str:_} = field;
        let ComponentData {group_name:_, point_name, write_ref_name, read_ref_name:_, is_must, c_type:_} = match mark {
            FieldMark::Component(data)  => data,
            _ => {
                field_creates.push(quote! {
                    #key: <#ty>::default()
                });
                continue;
            },
        };
        if is_must == &true {
            field_creates.push(quote! {
                #key: #point_name::create(&mut groups.#key, &p)
            });
            field_create_notifys.push(quote! {
                #write_ref_name::new(value.owner.#key.clone(), groups.#key.to_usize(), self.mgr).recursive_create_notify();
            });
        }else {
            field_creates.push(quote! {
                #key: #point_name(0)
            });
        }
        field_destroys.push(quote! {
            #write_ref_name::new(value.owner.#key.clone(), groups.#key.to_usize(), self.mgr).destroy();
        });
       
    }

    let mut destroy1 = quote! {};
    if field_destroys.len() > 0 {
        destroy1 = quote! {
            {
                let value =  groups._group.get(&self.point);
                #(#field_destroys)*
            }
        }
    }

    quote! {
        impl #p_name{
            fn create<M: ComponentMgr>(groups: &mut #g_name<M>, parent: &usize) -> #p_name{
                let v: #name = unsafe{uninitialized()};
                let p = groups._group.insert(v, parent.clone());
                let v1 = #name {
                    #(#field_creates),*
                };
                unsafe{write(&mut groups._group.get_mut(&p).owner as *mut #name, v1)};
                p
            }
        }

        impl<'a, M: ComponentMgr> #w_r_name<'a, M>{
            pub fn create(parent: &usize, group: usize, mgr: &'a mut M) -> #w_r_name<'a, M>{
                let groups = #g_name::<M>::from_usize_mut(group);
                let point = #p_name::create(groups, parent);
                let mut r = #w_r_name::new(point, group, mgr);
                r.recursive_create_notify();
                r
            }

            pub fn recursive_create_notify(&mut self){
                let groups = #g_name::<M>::from_usize(self.groups);
                let parent = {
                    let value = groups._group.get(&self.point);
                    #(#field_create_notifys)*
                    value.parent
                };
                let handlers = groups._group.get_handlers();
                notify(Event::Create{point: self.point.clone(), parent: parent}, &handlers.borrow(), &mut self.mgr);
            }

            pub fn destroy(&mut self){
                if self.point.0 > 0 {
                    let groups = #g_name::<M>::from_usize_mut(self.groups);
                    let parent = groups._group.get(&self.point).parent.clone();
                    let handlers = groups._group.get_handlers();
                    notify(Event::Delete{point: self.point.clone(), parent: parent}, &handlers.borrow(), &mut self.mgr);
                    #destroy1
                    groups._group.remove(&self.point);
                }
            }
        }
    }
}