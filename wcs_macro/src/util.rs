use quote::ToTokens;

pub fn is_child(field: &syn::Field) -> bool{
    let f_name = field.ty.clone().into_token_stream().to_string();
    if f_name.ends_with("Point") {
        true
    }else {
        false
    }
}

pub fn ident(sym: &str) -> syn::Ident {
    syn::Ident::new(sym, quote::__rt::Span::call_site())
}

pub fn group_name(name: String) -> syn::Ident {
    ident(&(name + "Group"))
}

pub fn point_name(name: String) -> syn::Ident {
    ident(&(name + "Point"))
}

pub fn ref_name(name: String) -> syn::Ident {
    ident(&(name + "Ref"))
}

pub fn set_name(name: &str) -> syn::Ident {
    ident(&("set_".to_string() + name))
}

pub fn get_name(name: &str) -> syn::Ident {
    ident(&("get_".to_string() + name))
}

pub fn get_name_mut(name: &str) -> syn::Ident {
    ident(&("get_".to_string() + name + "_mut"))
}

pub fn add_name(name: &str) -> syn::Ident {
    ident(&("add_".to_string() + name))
}

