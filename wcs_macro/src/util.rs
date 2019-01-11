use quote::ToTokens;

pub fn is_child(field: &syn::Field) -> bool{
    let attrs = &field.attrs;
    for a in attrs.iter(){
        if a.path.clone().into_token_stream().to_string().as_str() == "Component" {
            return true;
        }
    }
    false
}

pub fn is_ignore(field: &syn::Field) -> bool{
    let attrs = &field.attrs;
    for a in attrs.iter(){
        if a.path.clone().into_token_stream().to_string().as_str() == "Ignore" {
            return true;
        }
    }
    false
}

pub fn is_must(field: &syn::Field) -> bool{
    let attrs = &field.attrs;
    for a in attrs.iter(){
        if a.path.clone().into_token_stream().to_string().as_str() == "Must" {
            return true;
        }
    }
    false
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

pub fn read_ref_name(name: String) -> syn::Ident {
    ident(&(name + "ReadRef"))
}

pub fn write_ref_name(name: String) -> syn::Ident {
    ident(&(name + "WriteRef"))
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

// pub fn add_name(name: &str) -> syn::Ident {
//     ident(&("add_".to_string() + name))
// }

pub fn create_name(name: &str) -> syn::Ident {
    ident(&("create_".to_string() + name))
}

