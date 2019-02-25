use quote::quote;

use data::*;

pub fn impl_component_macro(ast: &syn::DeriveInput) -> quote::__rt::TokenStream {
    let name = &ast.ident;
    match &ast.data {
        syn::Data::Struct(s) => {
            impl_struct(name, s)
        },
        syn::Data::Enum(_) => {
            //impl_enum()
            panic!("xxxx")
        },
        syn::Data::Union(_) => panic!("xxxx"),
    }
}

pub fn impl_struct(name: &syn::Ident, s: &syn::DataStruct) -> quote::__rt::TokenStream {
    let mut arr = Vec::new();
    let fields = match &s.fields {
        syn::Fields::Named(f) => {
            Fields::from(&f.named, FieldsType::Named, |_f, _i|{true})
        },
        syn::Fields::Unnamed(f) => {
            Fields::from(&f.unnamed, FieldsType::Named, |_f, _i|{true})
        },
        syn::Fields::Unit => panic!("xxxx")
    };
    arr.push(def_ref(name));
    arr.push(impl_struct_ref(name, &fields));
    arr.push(component_group_tree(name, &fields));
    arr.push(component_impl_create(name, &fields));
    quote! {
        #(#arr)*
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
//             arr.push(def_id(name));
//             arr.push(impl_struct_id(name, fields));
//             arr.push(component_group_tree(name, fields));
//             quote! {
//                 #(#arr)*
//             }
//         },
//         syn::Fields::Unnamed(f) => panic!("xxxx"),
//         syn::Fields::Unit => panic!("xxxx")
//     }
// }

pub fn def_ref(name: &syn::Ident) -> quote::__rt::TokenStream {
    // let id = id_name(name.to_string());
    let read_ref = read_ref_name(name.to_string());
    let write_ref = write_ref_name(name.to_string());
    let group = group_name(name.to_string());
    quote! {
        // #[derive(Clone, Default, Debug)]
        // pub struct #id(pub usize);
        
        pub struct #read_ref<'a, M: ComponentMgr>{
            pub id: usize,
            groups: &'a #group<M>,
        }

        pub struct #write_ref<'a, M: ComponentMgr>{
            pub id: usize,
            groups: usize,
            mgr: &'a mut M,
        }
    }
}

pub fn impl_ref(name: &syn::Ident, readref_impls: &Vec<quote::__rt::TokenStream>, writeref_impls: &Vec<quote::__rt::TokenStream>) -> quote::__rt::TokenStream {
    let group = group_name(name.to_string());
    let read_reff = read_ref_name(name.to_string());
    let write_reff = write_ref_name(name.to_string());
    quote! {
        impl<'a, M: ComponentMgr> #read_reff<'a, M>{
            #(#readref_impls)*

            pub fn new(p: usize, g: &#group<M>) -> #read_reff<M>{
                #read_reff{
                    id: p,
                    groups: g,
                }
            }
        }

        impl<'a, M: ComponentMgr> Deref for #read_reff<'a, M>{
            type Target = usize;
            fn deref(&self) -> &usize{
                &self.id
            }
        }

        impl<'a, M: ComponentMgr> #write_reff<'a, M>{
            #(#writeref_impls)*

            pub fn modify<F: FnOnce(&mut #name) -> bool>(&mut self, m: F) {
                let groups = #group::<M>::from_usize_mut(self.groups);
                let handlers = groups._group.get_handlers();
                let mut elem = groups._group.get_mut(self.id);
                if m(&mut elem) {
                    notify(Event::ModifyField{
                        id: self.id.clone(),
                        parent: elem.parent,
                        field: ""
                    }, &handlers.borrow(), &mut self.mgr);
                }
            }

            pub fn new(p: usize, g: usize, m: &mut M) -> #write_reff< M>{
                #write_reff{
                    id: p,
                    groups: g,
                    mgr: m,
                }
            }
        }

        impl<'a, M: ComponentMgr> Deref for #write_reff<'a, M>{
            type Target = usize;
            fn deref(&self) -> &usize{
                &self.id
            }
        }
    }
}

pub fn impl_struct_ref(name: &syn::Ident, fields: &Fields) -> quote::__rt::TokenStream {
    let mut readref_impls = Vec::new();
    let mut writeref_impls = Vec::new();
    
    for f in fields.data.iter(){
        readref_impls.push(impl_struct_readref_fun( f));
        writeref_impls.push(impl_struct_writeref_fun(name, f));
    }

    impl_ref(name, &readref_impls, &writeref_impls)
}

pub fn impl_struct_readref_fun(field: &Field) -> quote::__rt::TokenStream {
    let Field{key, ty, set_name:_, get_name, get_mut_name:_, ty_name:_, mark, key_str:_} = field;
    match mark {
        FieldMark::Component(data) => {
            let ComponentData {group_name:_, id_name:_, write_ref_name:_, read_ref_name, is_must:_,c_type:_} = data;
            quote! {
            pub fn #get_name(&self) -> #read_ref_name<M>{
                    // let p = self.id.#get_name(self.groups).clone();
                    #read_ref_name::new(self.groups._group.get(self.id).#get_name().clone(), &self.groups.#key)
                }
            }
        },
        FieldMark::EnumComponent(data) => {
            let ComponentData {group_name:_, id_name:_, write_ref_name:_, read_ref_name, is_must:_, c_type:_} = data;
            quote! {
                pub fn #get_name(&self) -> #read_ref_name<M>{
                    #read_ref_name::new(self.groups._group.get(self.id).#get_name().clone(), &self.groups.#key)
                }
            }
        },
        FieldMark::Data => {
            quote! {
                pub fn #get_name(&self) -> &#ty{
                    unsafe{&*(self.groups._group.get(self.id).#get_name() as *const #ty)}
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
            let ComponentData {group_name:_, id_name:_, write_ref_name, read_ref_name, is_must:_, c_type} = data;
            quote! {
                pub fn #set_name(&mut self, value: #c_type){
                    let groups = #group::<M>::from_usize_mut(self.groups);
                    let (parent, new_id) = {
                        let elem = groups._group.get_mut(self.id);
                        
                        //销毁
                        {
                            let old = elem.#get_name().clone();
                            let mut old_ref = #write_ref_name::<M>::new(old, groups.#key.to_usize(), &mut self.mgr);
                            old_ref.destroy(); 
                        }

                        let index = groups.#key._group.insert(value, self.id);
                        elem.#set_name(index.clone());
                        (elem.parent, index)
                    };
   
                    // let parent = self.id.#set_name(value, groups);
                    let handlers = groups._group.get_handlers();
                    //创建事件
                    {
                        let mut v_ref = #write_ref_name::<M>::new(new_id, groups.#key.to_usize(), &mut self.mgr);
                        v_ref.set_parent(self.id); // 递归设置parent
                        v_ref.create_notify();
                    }
                    
                    // notify(Event::Create{
                    //     id: new_id,
                    //     parent: parent,
                    // }, &handlers1.borrow(), &mut self.mgr);

                    //修改事件
                    notify(Event::ModifyField{
                        id: self.id.clone(),
                        parent: parent,
                        field: #key_str
                    }, &handlers.borrow(), &mut self.mgr);
                }

                pub fn #get_name(&self) -> #read_ref_name<M>{
                    let groups = #group::<M>::from_usize(self.groups);
                    // let p = self.id.#get_name(groups).clone();
                    #read_ref_name::new(groups._group.get(self.id).#get_name().clone(), &groups.#key)
                }

                pub fn #get_mut_name(&mut self) -> #write_ref_name<M>{
                    let groups = #group::<M>::from_usize_mut(self.groups);
                    // let p = self.id.#get_name(groups).clone();
                    #write_ref_name::new(groups._group.get(self.id).#get_name().clone(), groups.#key.to_usize(), &mut self.mgr)
                }
            }
        },
        FieldMark::EnumComponent(data) => {
            let ComponentData {group_name:_, id_name, write_ref_name, read_ref_name, is_must:_, c_type} = data;
            quote! {
                pub fn #set_name(&mut self, value: #c_type){
                    let groups = #group::<M>::from_usize_mut(self.groups);

                    let parent = {
                        let elem = groups._group.get_mut(self.id);

                        //销毁
                        {
                            let old = elem.#get_name().clone();
                            let mut old_ref = #write_ref_name::<M>::new(old, groups.#key.to_usize(), &mut self.mgr);
                            old_ref.destroy(); 
                        }

                        let new_id = #id_name::_set(&mut groups.#key, value, self.id);
                        elem.#set_name(new_id.clone()); // 递归设置parent
                        let mut new_write = #write_ref_name::<M>::new(new_id, groups.#key.to_usize(), &mut self.mgr);
                        new_write.set_parent(self.id);
                        new_write.create_notify();
                        elem.parent
                    };

                    let handlers = groups._group.get_handlers();
                    //修改事件
                    notify(Event::ModifyField{
                        id: self.id.clone(),
                        parent: parent,
                        field: #key_str
                    }, &handlers.borrow(), &mut self.mgr);
                }

                pub fn #get_name(&self) -> #read_ref_name<M>{
                    let groups = #group::<M>::from_usize(self.groups);
                    #read_ref_name::new(groups._group.get(self.id).#get_name().clone(), &groups.#key)
                }

                pub fn #get_mut_name(&mut self) -> #write_ref_name<M>{
                    let groups = #group::<M>::from_usize_mut(self.groups);
                    #write_ref_name::new(groups._group.get(self.id).#get_name().clone(), groups.#key.to_usize(), &mut self.mgr)
                }
            }
        },
        FieldMark::Data => {
            quote! {
                pub fn #set_name(&mut self, value: #ty){
                    let groups = #group::<M>::from_usize_mut(self.groups);
                    let parent = {
                        let elem = groups._group.get_mut(self.id);
                        elem.#set_name(value);
                        elem.parent
                    };
                    //let parent = self.id.#set_name(value, groups);
                    let handlers = groups._group.get_handlers();
                    notify(Event::ModifyField{
                        id: self.id.clone(),
                        parent: parent,
                        field: #key_str
                    }, &handlers.borrow(), &mut self.mgr);
                }

                pub fn #get_name(&self) -> &#ty{
                    let groups = #group::<M>::from_usize(self.groups);
                    unsafe{&*(groups._group.get(self.id).#get_name() as *const #ty)}
                }

                pub fn #get_mut_name(&self) -> &mut #ty{
                    let groups = #group::<M>::from_usize_mut(self.groups);
                    unsafe{&mut *(groups._group.get_mut(self.id).#get_mut_name() as *mut #ty)}
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
        let ComponentData {group_name, id_name:_, write_ref_name:_, read_ref_name:_, is_must:_, c_type:_} = match mark {
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
    // let id_name = id_name(name.to_string());

    quote! {
        pub struct #group_name<M: ComponentMgr>{
            pub _group: ComponentGroup<#name, M>,
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
    let mut field_set_parent = Vec::new();
    // let p_name = id_name(name.to_string());
    let w_r_name = write_ref_name(name.to_string());
    let g_name = group_name(name.to_string());
    let Fields {ty:_, data} = fields;
    for field in data.iter(){
        let Field{key, ty, set_name:_, get_name:_, get_mut_name:_, ty_name: _, mark, key_str:_} = field;
        let ComponentData {group_name:_, id_name, write_ref_name, read_ref_name:_, is_must, c_type:_} = match mark {
            FieldMark::Component(data)  => data,
            FieldMark::EnumComponent(data)  => data,
            _ => {
                field_creates.push(quote! {
                    #key: <#ty>::default()
                });
                continue;
            },
        };
        if is_must == &true {
            field_creates.push(quote! {
                #key: #id_name::create(&mut groups.#key, &p)
            });
        }else {
            field_creates.push(quote! {
                #key: #id_name(0)
            });
        }
        field_create_notifys.push(quote! {
            #write_ref_name::new(value.#key.clone(), groups.#key.to_usize(), self.mgr).create_notify();
        });
        field_destroys.push(quote! {
            #write_ref_name::new(value.#key.clone(), groups.#key.to_usize(), self.mgr).destroy();
        });

        field_set_parent.push(quote! {
            #write_ref_name::new(value.#key.clone(), groups.#key.to_usize(), self.mgr).set_parent(parent);
        })
    }

    let mut destroy1 = quote! {};
    if field_destroys.len() > 0 {
        destroy1 = quote! {
            {
                let value =  groups._group.get(self.id);
                #(#field_destroys)*
            }
        }
    }

    quote! {
        // impl #p_name{
        //     fn create<M: ComponentMgr>(groups: &mut #g_name<M>, parent: &usize) -> #p_name{
        //         let v: #name = unsafe{uninitialized()};
        //         let p = groups._group.insert(v, parent.clone());
        //         let v1 = #name {
        //             #(#field_creates),*
        //         };
        //         unsafe{write(&mut groups._group.get_mut(&p).owner as *mut #name, v1)};
        //         p
        //     }
        // }

        impl<'a, M: ComponentMgr> #w_r_name<'a, M>{
            // pub fn create(parent: &usize, group: usize, mgr: &'a mut M) -> #w_r_name<'a, M>{
            //     let groups = #g_name::<M>::from_usize_mut(group);
            //     let id = #p_name::create(groups, parent);
            //     let mut r = #w_r_name::new(id, group, mgr);
            //     r.recursive_create_notify();
            //     r
            // }

            pub fn create_notify(&mut self){
                if self.id == 0 {
                    return;
                }
                let groups = #g_name::<M>::from_usize(self.groups);
                let parent = {
                    let value = groups._group.get(self.id);
                    #(#field_create_notifys)*
                    value.parent
                };
                let handlers = groups._group.get_handlers();
                notify(Event::Create{id: self.id.clone(), parent: parent}, &handlers.borrow(), &mut self.mgr);
            }

            //递归设置parent
            pub fn set_parent(&mut self, parent: usize){
                let mut groups = #g_name::<M>::from_usize_mut(self.groups);
                {
                    let mut value = groups._group.get_mut(self.id);
                    value.parent = parent;
                    let parent = self.id;
                    #(#field_set_parent)*
                }
            }

            pub fn destroy(&mut self){
                if self.id > 0 {
                    let groups = #g_name::<M>::from_usize_mut(self.groups);
                    let parent = groups._group.get(self.id).parent.clone();
                    let handlers = groups._group.get_handlers();
                    notify(Event::Delete{id: self.id.clone(), parent: parent}, &handlers.borrow(), &mut self.mgr);
                    #destroy1
                    groups._group.remove(self.id);
                }
            }
        }
    }
}