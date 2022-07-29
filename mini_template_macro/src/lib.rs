mod create_modifier;
mod derive_value_container;

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

#[proc_macro_derive(ValueContainer, attributes(name))]
pub fn derive_value_container(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::DeriveInput);
    let result = derive_value_container::derive_value_container(item);
    match result {
        Ok(o) => o,
        Err(e) => e.to_compile_error(),
    }
    .into()
}