use proc_macro::TokenStream;

/*
* js代码生成器使用的导出属性
*/
#[proc_macro_attribute]
pub fn pi_js_export(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}