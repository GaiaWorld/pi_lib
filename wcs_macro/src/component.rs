use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;

use util::*;

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
            let mut arr = Vec::new();
            let fields = &f.named;
            arr.push(def_point(name));
            arr.push(impl_struct_point(name, fields));
            arr.push(component_group_tree(name, fields));
            quote! {
                #(#arr)*
            }
        },
        syn::Fields::Unnamed(_) => panic!("xxxx"),
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
    let reff = ref_name(name.to_string());
    let group = group_name(name.to_string());
    quote! {
        #[derive(Clone, Default)]
        pub struct #point(pub usize);

        pub struct #reff<M: ComponentMgr>{
            point: #point,
            groups: Rc<RefCell<#group<M>>>,
        }
    }
}

pub fn impl_point(name: &syn::Ident, point_impls: &Vec<quote::__rt::TokenStream>, ref_impls: &Vec<quote::__rt::TokenStream>) -> quote::__rt::TokenStream {
    let point = point_name(name.to_string());
    let group = group_name(name.to_string());
    let reff = ref_name(name.to_string());
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

        impl #point{
            #(#point_impls)*
        }

        impl<M: ComponentMgr> #reff<M>{
            #(#ref_impls)*

            pub fn new(p: #point, g: Rc<RefCell<#group<M>>>) -> #reff<M>{
                #reff{
                    point: p,
                    groups: g,
                }
            }
        }
    }
}

pub fn impl_struct_point(name: &syn::Ident, fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> quote::__rt::TokenStream {
    let mut point_impls = Vec::new();
    let mut ref_impls = Vec::new();
    
    for f in fields.iter(){
        point_impls.push(impl_struct_point_fun(name, f));
        ref_impls.push(impl_struct_pointref_fun(f))
    }

    impl_point(name, &point_impls, &ref_impls)
}

pub fn impl_struct_point_fun(name: &syn::Ident, field: &syn::Field) -> quote::__rt::TokenStream {
    let group = group_name(name.to_string());
    let field_name_str = match &field.ident {
        Some(ref i) => i.to_string(),
        None => panic!("no fieldname"),
    };
    let set = set_name(&field_name_str);
    let get = get_name(&field_name_str);
    let get_mut = get_name_mut(&field_name_str);
    let field_ty_str = field.ty.clone().into_token_stream().to_string();
    //let field_ty_point = point_name(field_ty.clone());
    let field_name = ident(&field_name_str);
    let is_child = is_child(field);

    if is_child {
        let field_ty= ident(unsafe{field_ty_str.get_unchecked(0..field_ty_str.len()-5)});
        let field_ty_point = ident(&field_ty_str);
        quote! {
            pub fn #set<M: ComponentMgr>(&mut self, value: #field_ty, groups: &mut #group<M>){
                let index = groups.#field_name.borrow_mut()._group.insert(value, self.0.clone());
                groups._group.get_mut(self).#field_name = index;
                groups._group.notify(EventType::ModifyField(self.clone(), #field_name_str));
            }

            pub fn #get<M: ComponentMgr>(&self, groups: &#group<M>) -> #field_ty_point{
                groups._group.get(self).#field_name.clone()
            }
        }
    }else {
        let field_ty = &field.ty;
        quote! {
            pub fn #set<M: ComponentMgr>(&mut self, value: #field_ty, groups: &mut #group<M>){
                groups._group.get_mut(self).#field_name = value;
                groups._group.notify(EventType::ModifyField(self.clone(), #field_name_str));
            }

            pub fn #get<'a, M: ComponentMgr>(&self, groups: &'a #group<M>) -> &'a #field_ty{
                &(groups._group.get(self).#field_name)
            }

            pub fn #get_mut<'a, M: ComponentMgr>(&mut self, groups: &'a mut #group<M>) -> &'a mut #field_ty{
                &mut (groups._group.get_mut(self).#field_name)
            }
        }
    }
}

pub fn impl_struct_pointref_fun(field: &syn::Field) -> quote::__rt::TokenStream {
    let field_ty_str = field.ty.clone().into_token_stream().to_string();
    let field_name_str = match &field.ident {
        Some(ref i) => i.to_string(),
        None => panic!("no fieldname"),
    };
    let set = set_name(&field_name_str);
    let get = get_name(&field_name_str);
    let get_mut = get_name_mut(&field_name_str);
    let field_name = ident(&field_name_str);
    let is_child = is_child(field);

    if is_child {
        let field_ty= ident(unsafe{field_ty_str.get_unchecked(0..field_ty_str.len()-5)});
        let field_ty_ref = ref_name(unsafe{field_ty_str.get_unchecked(0..field_ty_str.len()-5)}.to_string());
        quote! {
            pub fn #set(& mut self, value: #field_ty){
                self.point.#set(value, &mut self.groups.borrow_mut());
            }

            pub fn #get(&self) -> #field_ty_ref<M>{
                let p = self.point.#get(&self.groups.borrow()).clone();
                #field_ty_ref::new(p, self.groups.borrow().#field_name.clone())
            }
        }
    }else {
        let field_ty = &field.ty;
        quote! {
            pub fn #set(&mut self, value: #field_ty){
                self.point.#set(value, &mut self.groups.borrow_mut());
            }

            pub fn #get(&self) -> &#field_ty{
                unsafe{&*(self.point.#get(&self.groups.borrow()) as *const #field_ty)}
            }

            pub fn #get_mut(&mut self) -> &mut #field_ty{
                unsafe{&mut *(self.point.#get_mut(&mut self.groups.borrow_mut()) as *mut #field_ty)}
            }
        }
    }
}

pub fn component_group_tree(name: &syn::Ident, fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> quote::__rt::TokenStream {
    let mut field_types = Vec::new();
    let mut field_news = Vec::new();
    let mut set_mgrs = Vec::new();
    for field in fields.iter(){
        if !is_child(field){
            continue;
        }

        let field_name = match &field.ident {
            Some(ref i) => ident(&i.to_string()),
            None => panic!("no fieldname"),
        };
        let field_ty_str = field.ty.clone().into_token_stream().to_string();
        let field_ty_group: syn::Ident = group_name(unsafe{field_ty_str.get_unchecked(0..field_ty_str.len() -5)}.to_string());
        field_types.push(quote! {
            pub #field_name: Rc<RefCell<#field_ty_group<M>>>,
        });
        field_news.push(quote! {
            #field_name: Rc::new(RefCell::new(#field_ty_group::new())),
        });
        set_mgrs.push(quote! {
            self.#field_name.borrow_mut().set_mgr(mgr.clone());
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

            fn set_mgr(&mut self, mgr: Weak<RefCell<Self::C>>){
                #(#set_mgrs)*
                self._group.set_mgr(mgr);
            }
        }
    }
}
