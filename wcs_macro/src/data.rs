use quote::ToTokens;

pub fn is_component(field: &syn::Field) -> bool{
    let attrs = &field.attrs;
    for a in attrs.iter(){
        if a.path.clone().into_token_stream().to_string().as_str() == "component" {
            return true;
        }
    }
    false
}

pub fn is_enum_component(field: &syn::Field) -> bool{
    let attrs = &field.attrs;
    for a in attrs.iter(){
        if a.path.clone().into_token_stream().to_string().as_str() == "enum_component" {
            return true;
        }
    }
    false
}

pub fn component_name(field: &syn::Field) -> String{
    let attrs = &field.attrs;
    for a in attrs.iter(){
        if a.path.clone().into_token_stream().to_string().as_str() == "enum_component" || a.path.clone().into_token_stream().to_string().as_str() == "component" {
            let inner = a.tts.to_string();
            match inner.get(1..inner.len() - 1) {
                Some(r) => return r.trim().to_string(),
                None => panic!("component_name error: {}", inner),
            }
        }
    }
    panic!("component_name error");
}

pub fn is_ignore(field: &syn::Field) -> bool{
    let attrs = &field.attrs;
    for a in attrs.iter(){
        if a.path.clone().into_token_stream().to_string().as_str() == "ignore" {
            return true;
        }
    }
    false
}

pub fn is_must(field: &syn::Field) -> bool{
    let attrs = &field.attrs;
    for a in attrs.iter(){
        if a.path.clone().into_token_stream().to_string().as_str() == "must" {
            return true;
        }
    }
    false
}

pub fn is_base_type(ty: &syn::Type) -> bool{
    let s = ty.clone().into_token_stream().to_string();
    if &s =="bool" || &s =="String" || &s =="f32" || &s =="f64" || &s =="i8" || &s =="i16" || &s =="i32" || &s =="i64" || &s =="i128" || &s =="u8" || &s =="u16" || &s =="u32" || &s =="u64" || &s =="u128" || &s =="usize" || &s =="isize" {
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

pub fn id_name(name: String) -> syn::Ident {
    ident(&(name + "Id"))
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

pub fn add_name(name: &str) -> syn::Ident {
    ident(&("add_".to_string() + name))
}

// pub fn create_name(name: &str) -> syn::Ident {
//     ident(&("create_".to_string() + name))
// }


#[derive(Clone)]
pub struct Field{
    pub key: syn::Ident, //字段名称
    pub key_str: String, //字段名称
    pub ty: syn::Type, //字段类型
    pub set_name: syn::Ident, //set方法名称
    pub get_name: syn::Ident, //get方法名称
    pub get_mut_name: syn::Ident,
    pub ty_name: syn::Type, //类型， 不包含泛型
    pub mark: FieldMark, // 字段标记
}

#[derive(Clone)]
pub struct ComponentData{
    pub group_name: syn::Ident, //组名
    pub id_name: syn::Ident, //指针名
    pub write_ref_name: syn::Ident, //写引用名称
    pub read_ref_name: syn::Ident, //读引用名称
    pub is_must: bool, //是否为必须的字段
    pub c_type: syn::Ident, //组件名称
}

impl ComponentData {
    pub fn from(name: String, is_must: bool) -> ComponentData{
        ComponentData{
            group_name: group_name(name.clone()),
            id_name: id_name(name.clone()),
            write_ref_name: write_ref_name(name.clone()),
            read_ref_name: read_ref_name(name.clone()),
            is_must: is_must,
            c_type: ident(&name)
        }
    }
}

impl Field {
    pub fn from(field: syn::Field, index: usize) -> Field{
        let mark = if is_component(&field) || is_enum_component(&field) {
            let field_ty_str = component_name(&field);
            let c_n = ComponentData::from(field_ty_str, is_must(&field));

            if is_component(&field) {
                FieldMark::Component(c_n)
            }else {
                FieldMark::EnumComponent(c_n)
            }
        }else {
            (FieldMark::Data)
        };

        let key_str = match field.ident {
            Some(i) => i.to_string(),
            None => "i_".to_string() + index.to_string().as_str(),
        } .to_string();

        let mut type_name = field.ty.clone();
        match &mut type_name {
            syn::Type::Path(ref mut p) => {
                for v in p.path.segments.iter_mut(){
                    v.arguments = syn::PathArguments::None;
                }
            },
            _ => (),
        }

        Field {
            key: ident(&key_str),
            key_str: key_str.clone(),
            ty: field.ty.clone(),
            set_name: set_name(&key_str),
            get_name: get_name(&key_str),
            get_mut_name: get_name_mut(&key_str),
            ty_name: type_name,
            mark: mark,
        }
    }
}

#[derive(Clone)]
pub struct Fields{
    pub ty: FieldsType,
    pub data: Vec<Field>,
}

impl Fields {
    pub fn from<F: Fn(&syn::Field, usize) -> bool >(fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, ty: FieldsType, filter: F) -> Fields{
        let mut data = Vec::new();
        let mut i = 0;
        for field in fields.iter(){
            if !filter(field, i) {
                continue;
            }
            

            if is_ignore(field){
                continue;
            }

            data.push(Field::from(field.clone(), i));
            i += 1;
        }

        Fields{
            ty,
            data
        }

    }
}

// #[derive(Clone)]
// pub struct VariantFields{
//     pub ty: FieldsType,
//     pub keys: Vec<syn::Ident>, //字段名称
//     pub key_strs: Vec<String>, //字段名称
//     pub tys: Vec<syn::Type>, //字段类型
//     pub set_names: Vec<syn::Ident>, //set方法名称
//     pub get_names: Vec<syn::Ident>, //get方法名称
//     pub get_mut_names: Vec<syn::Ident>,
//     pub ty_names: Vec<syn::Type>, //类型， 不包含泛型
//     pub group_names: Vec<syn::Ident>, //组名
//     pub id_names: Vec<syn::Ident>, //指针名
//     pub write_ref_names: Vec<syn::Ident>, //写引用名称
//     pub read_ref_names: Vec<syn::Ident>, //读引用名称
// }

// impl VariantFields {
//     pub fn from(fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, ty: FieldsType) -> VariantFields{
//         let mut i = 0;
//         let mut keys: Vec<syn::Ident> = Vec::new(); //字段名称
//         let mut key_strs: Vec<String> = Vec::new(); //字段名称
//         let mut tys: Vec<syn::Type> = Vec::new(); //字段类型
//         let mut set_names: Vec<syn::Ident> = Vec::new(); //set方法名称
//         let mut get_names: Vec<syn::Ident> = Vec::new(); //get方法名称
//         let mut get_mut_names: Vec<syn::Ident> = Vec::new();
//         let mut ty_names: Vec<syn::Type> = Vec::new(); //类型， 不包含泛型
//         let mut group_names: Vec<syn::Ident> = Vec::new(); //组名
//         let mut id_names: Vec<syn::Ident> = Vec::new(); //指针名
//         let mut write_ref_names: Vec<syn::Ident> = Vec::new(); //写引用名称
//         let mut read_ref_names: Vec<syn::Ident> = Vec::new(); //读引用名称
//         for field in fields.iter(){
//             if is_ignore(field){
//                 continue;
//             }
//             let Field{key, key_str, ty, set_name, get_name, get_mut_name, ty_name, mark: _} = Field::from(field.clone(), i);
//             let field_ty_str = ty.clone().into_token_stream().to_string();
//             let ComponentData{group_name, id_name, write_ref_name, read_ref_name, is_must:_, c_type: _} = ComponentData::from(field_ty_str, true);
//             keys.push(key);
//             key_strs.push(key_str);
//             tys.push(ty);
//             set_names.push(set_name);
//             get_names.push(get_name);
//             get_mut_names.push(get_mut_name);
//             ty_names.push(ty_name);
//             group_names.push(group_name);
//             id_names.push(id_name);
//             write_ref_names.push(write_ref_name);
//             read_ref_names.push(read_ref_name);
//             i += 1;
//         }

//         VariantFields{
//             ty,
//             keys,
//             key_strs,
//             tys,
//             set_names,
//             get_names,
//             get_mut_names,
//             ty_names,
//             group_names,
//             write_ref_names,
//             read_ref_names,
//             id_names,
//         }

//     }
// }

#[derive(Clone)]
pub enum FieldMark{
    Component(ComponentData),
    EnumComponent(ComponentData),
    Data,
}

#[derive(Clone)]
pub enum FieldsType{
    Named,
    Unnamed,
}

#[derive(Clone)]
pub struct Variant{
    pub key: syn::Ident, //字段名称
    pub fields: Fields, //字段名称
}

impl Variant {
    pub fn from(variant: &syn::Variant) -> Variant{
        let fields = {
            match &variant.fields {
                syn::Fields::Named(field) => Fields::from(&field.named, FieldsType::Named, |_, _|{true}),
                syn::Fields::Unnamed(field) => Fields::from(&field.unnamed, FieldsType::Unnamed, |_, _|{true}),
                _ => panic!("enum error"),
            }
        };
        Variant{
            key: variant.ident.clone(),
            fields: fields
        }
    }
}

pub struct Variants{
    pub data: Vec<Variant>,
}

impl Variants {
    pub fn from(variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> Variants{
        let mut data = Vec::new();
        for variant in variants.iter(){
            data.push(Variant::from(&variant));
        }

        Variants{data}
    }
}

// pub struct StructData{
//     pub name: syn::Ident,
//     pub fields: Fields,
//     pub component_data: ComponentData,
// }

// impl StructData {
//     pub fn from(s: &syn::DataStruct, name: &syn::Ident) -> StructData{
//         StructData{
//             name: name.clone(),
//             component_data: ComponentData::from(name.to_string(), false),
//             fields: match s.fields {
//                 syn::Fields::Named(ref data)=> Fields::from(&data.named, FieldsType::Named, |_r, _r1|{true} ),
//                 syn::Fields::Unnamed(ref data) => Fields::from(&data.unnamed, FieldsType::Unnamed, |_r, _r1|{true}),
//                 _ => panic!("struct error"),
//             } 
//         }

//     }
// }

pub struct EnumData{
    pub name: syn::Ident,
    pub component_data: ComponentData,
    pub variants: Variants,
}


impl EnumData {
    pub fn from(s: &syn::DataEnum, name: &syn::Ident) -> EnumData{
        EnumData{
            name: name.clone(),
            component_data: ComponentData::from(name.to_string(), false),
            variants: Variants::from(&s.variants),
        }
    }
}