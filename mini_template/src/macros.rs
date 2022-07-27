pub use mini_template_macro::create_modifier;
pub use mini_template_macro::ValueContainer;

/// Creates a new modifier based on a method.
/// This macro is usually used to create new template modifiers. The method header of the resulting
/// method will look different from the given header.
///
/// Possible parameter types are:
///
/// &str, String, bool, f64, isize, i32, usize, u32
///
/// # Example
/// ```
/// use mini_template::value::Value;
/// use mini_template::fn_as_modifier;
///
/// fn repeat_n_times(s: &str, n: usize) -> String {
///     let mut result = String::new();
///     for _ in 0..n {
///         result.push_str(s)
///     }
///     result
/// }
///
/// fn_as_modifier!(
///     fn repeat_n_times_modifier(s: &str, n: usize) -> String => repeat_n_times
/// );
///
/// assert_eq!(
///     repeat_n_times_modifier(&Value::String("17".to_owned()), vec![&Value::Number(2.)]),
///     Ok(Value::String("1717".to_owned()))
/// );
/// ```
/// # Warning
/// The variants `default` and `try_into` are for internal use only and are subject to change without further notice.
#[macro_export]
macro_rules! fn_as_modifier {
    (fn $modifier_name: ident ($first_name:ident: $first_t: ty $($(,$name: ident: $t: ty $(= $default: expr)?)+)?) -> $return: ty => $func: path) => {
        #[allow(unused_variables)]
        pub fn $modifier_name(value: &$crate::value::Value, args: Vec<&$crate::value::Value>) -> $crate::modifier::error::Result<$crate::value::Value> {
            use $crate::modifier::error::Error;

            let $first_name: $first_t = fn_as_modifier!(try_into value: $first_t);

            $(
                let mut args = args.into_iter();
                $(
                    let $name: $t = match args.next() {
                        Some($name) => fn_as_modifier!(try_into $name: $t),
                        None => fn_as_modifier!(default_value $name $($default)?)
                    };
                )+
            )?

            let result = $func($first_name $($(,$name)+)?);
            Ok(result.into())
        }
    };
    (default_value $arg_name: ident) => {
        return Err(Error::MissingArgument{argument_name: stringify!($arg_name)})
    };
    (default_value $arg_name: ident $default: tt) => {
        $default
    };
    (try_into $value: ident: $type: ty) => {
        match $value.try_into() {
            Ok(inner) => inner,
            Err(e) => return Err(Error::Type{value: $value.to_string(), type_error: e})
        }
    }
}

#[macro_export]
macro_rules! value_iter {
    (
        $($ident:literal: $value: expr),*
    ) => {
        [
            $(
                ($crate::value::ident::Ident::try_from($ident).unwrap(), $value)
            ),*
        ]
    };
}
