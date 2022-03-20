mod create_modifier;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn create_modifier(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(_attr as syn::AttributeArgs);
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let result = create_modifier::create_modifier(attr, item.into());
    match result {
        Ok(o) => o,
        Err(e) => e.to_compile_error(),
    }
    .into()
}
