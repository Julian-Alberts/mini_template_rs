mod create_modifier;
mod derive_value_container;

use proc_macro::TokenStream;

///## With body
/// ```
/// use mini_template::value::Value;
/// use mini_template_macro::create_modifier;
///
/// #[create_modifier]
/// fn fizz_buzz(n: usize) -> String {
///     match (n % 3, n % 5) {
///         (0, 0) => String::from("FIZZBUZZ"),
///         (0, _) => String::from("FIZZ"),
///         (_, 0) => String::from("BUZZ"),
///         _ => n.to_string()
///     }
/// }
///
///
/// assert_eq!(
///     fizz_buzz(
///         &Value::Number(mini_template::value::Number::USize(3)),
///         Vec::default()
///     ),
///     Ok(Value::String(String::from("FIZZ")))
/// );
/// ```
/// ## Returns Result
/// ```
/// use mini_template::value::Value;
/// use mini_template_macro::create_modifier;
///
/// #[create_modifier(returns_result = true)]
/// fn as_usize(n: String) -> Result<usize, String> {
///     match n.parse() {
///         Ok(n) => Ok(n),
///         Err(_) => Err(format!("Can not convert {n} to usize"))
///     }
/// }
///
///
/// assert!(as_usize(&Value::String("17".to_owned()), Vec::default()).is_ok());
/// assert!(as_usize(&Value::String("Foo".to_owned()), Vec::default()).is_err());
/// ```
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
