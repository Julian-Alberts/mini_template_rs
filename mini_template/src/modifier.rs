#[cfg(feature = "regex")]
use {
    core::ops::{Add, Mul, Div, Sub},
    once_cell::sync::OnceCell,
    regex::Regex,
    std::{
        collections::{hash_map::DefaultHasher, HashMap},
        hash::Hash,
        sync::RwLock,
    },
};

use super::value::Value;
pub use error::*;

#[cfg(feature = "regex")]
static REGEX_CACHE: OnceCell<RwLock<HashMap<u64, Regex>>> = OnceCell::new();

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
/// use mini_template::create_modifier;
///
/// fn repeat_n_times(s: &str, n: usize) -> String {
///     let mut result = String::new();
///     for _ in 0..n {
///         result.push_str(s)
///     }
///     result
/// }
///
/// create_modifier!(
///     fn repeat_n_times_modifier(s: &str, n: usize) -> String => repeat_n_times
/// );
///
/// assert_eq!(
///     repeat_n_times_modifier(&Value::String("17".to_owned()), vec![&Value::Number(2.)]),
///     Ok(Value::String("1717".to_owned()))
/// );
/// ```
#[macro_export]
macro_rules! create_modifier {
    (fn $modifier_name: ident ($first_name:ident: $first_t: ty $($(,$name: ident: $t: ty $(= $default: expr)?)+)?) -> $return: ty => $func: path) => {
        #[allow(unused_variables)]
        pub fn $modifier_name(value: &$crate::value::Value, args: Vec<&$crate::value::Value>) -> $crate::modifier::error::Result<$crate::value::Value> {
            use $crate::modifier::error::Error;

            let $first_name: $first_t = create_modifier!(try_into value: $first_t);

            $(
                let mut args = args.into_iter();
                $(
                    let $name: $t = match args.next() {
                        Some($name) => create_modifier!(try_into $name: $t),
                        None => create_modifier!(default_value $name $($default)?)
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

pub type Modifier = dyn Fn(&Value, Vec<&Value>) -> Result<Value>;

#[mini_template_derive::create_modifier]
fn slice_modifier(input: String, start: usize, length: usize) -> String {
    let chars = input.chars().skip(start);
    chars.take(length).collect::<String>()
}

#[cfg(feature = "regex")]
#[mini_template_derive::create_modifier(returns_result = true, defaults::group = 0)]
fn match_modifier(input: String, regex: String, group: usize) -> std::result::Result<String, String> {
    with_regex_from_cache(regex, |regex| {
        match regex.captures(&input[..]) {
            Some(c) => match c.get(group) {
                Some(c) => c.as_str(),
                None => ""
            },
            None => ""
        }.to_owned()
    })
}

#[mini_template_derive::create_modifier(defaults::count = 0)]
fn replace_modifier(input: String, from: String, to: String, count: usize) -> String {
    if count == 0 {
        input.replace(&from[..], &to[..])
    } else {
        input.replacen(&from[..], &to[..], count)
    }
}

#[cfg(feature = "regex")]
#[mini_template_derive::create_modifier(defaults::count = 0, returns_result = true)]
fn replace_regex_modifier(input: String, regex: String, to: String, count: usize) -> std::result::Result<String, String> {
    with_regex_from_cache(regex, |regex| {
        regex.replacen(&input, count, to).to_string()
    })
}

create_modifier!(fn upper(input: &str) -> String => str::to_uppercase);

create_modifier!(fn lower(input: &str) -> String => str::to_lowercase);

create_modifier!(fn add(a: f64, b: f64) -> f64 => f64::add);

create_modifier!(fn sub(a: f64, b: f64) -> f64 => f64::sub);

create_modifier!(fn mul(a: f64, b: f64) -> f64 => f64::mul);

create_modifier!(fn div(a: f64, b: f64) -> f64 => f64::div);

create_modifier!(fn repeat(input: &str, n: usize) -> String => str::repeat);

#[cfg(feature = "regex")]
fn with_regex_from_cache<F, T>(regex: String, f: F) -> std::result::Result<T, String>
where
    F: FnOnce(&Regex) -> T,
{
    use std::hash::Hasher;
    let mut hasher = DefaultHasher::new();
    regex.hash(&mut hasher);
    let cache_key = hasher.finish();
    drop(hasher);

    let cache = REGEX_CACHE.get_or_init(Default::default);
    let cache_r = cache.read().unwrap();
    let result = match cache_r.get(&cache_key) {
        Some(r) => (f)(r),
        None => {
            drop(cache_r);
            let regex = match Regex::new(&regex) {
                Ok(r) => r,
                Err(r) => return Err(r.to_string()),
            };
            let result = f(&regex);
            let mut cache_w = cache.write().unwrap();
            cache_w.insert(cache_key, regex);
            result
        }
    };

    Ok(result)
}

#[cfg(feature = "regex")]
pub fn regex_cache_clear() {
    REGEX_CACHE.set(RwLock::default()).unwrap();
}

pub mod error {
    use std::fmt::Display;

    use crate::value::TypeError;

    pub type Result<T> = std::result::Result<T, Error>;
    #[derive(Debug, PartialEq)]
    pub enum Error {
        MissingArgument {
            argument_name: &'static str,
        },
        Type {
            value: String,
            type_error: TypeError,
        },
        Modifier(String),
    }

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::MissingArgument { argument_name } => {
                    write!(f, "Missing argument \"{}\"", argument_name)
                }
                Self::Type { value, type_error } => write!(
                    f,
                    "Can not convert {} to type {} value of type {} found",
                    value, type_error.expected_type, type_error.storage_type
                ),
                Self::Modifier(e) => write!(f, "{}", e.to_owned()),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::value::TypeError;

    use super::*;

    #[cfg(feature = "regex")]
    #[test]
    fn match_modifier() {
        let input = Value::String(String::from("My 2test2 string"));
        let regex = Value::String(String::from(r#"(\d[a-z]+\d) string"#));
        let invalid_regex = Value::String(String::from(r#"(\d[a-z]+\d string"#));
        let full_match = Value::Number(0.0);
        let group = Value::Number(1.0);
        let args = vec![&regex, &full_match];

        let result = super::match_modifier(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2 string"))));

        let args = vec![&regex];

        let result = super::match_modifier(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2 string"))));

        let args = vec![&regex, &group];
        let result = super::match_modifier(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2"))));

        let args = vec![&invalid_regex, &full_match];
        let result = super::match_modifier(&input, args);
        assert_eq!(
            result,
            Err(Error::Modifier(
                "regex parse error:\n    (\\d[a-z]+\\d string\n    ^\nerror: unclosed group"
                    .to_owned()
            ))
        )
    }

    #[test]
    fn slice_modifier() {
        let input = Value::String(String::from("Hello World!!!"));
        let start_in = Value::Number(6f64);
        let start_out = Value::Number(14f64);
        let length_5 = Value::Number(5f64);

        let args = vec![&start_in, &length_5];

        let result = super::slice_modifier(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("World"))));

        let args = vec![&start_out, &length_5];

        let result = super::slice_modifier(&input, args);
        assert_eq!(result, Ok(Value::String(String::from(""))))
    }

    #[test]
    fn missing_argument() {
        let input = Value::String(String::from("My test string"));
        let args = vec![];

        let result = super::slice_modifier(&input, args);
        assert_eq!(
            result,
            Err(Error::MissingArgument {
                argument_name: "start"
            })
        );
    }

    #[test]
    fn can_not_parse_argument() {
        let input = Value::String(String::from("My test string"));

        let string = Value::String(String::from(r#"string"#));

        let args = vec![&string];

        let result = super::slice_modifier(&input, args);
        assert_eq!(
            result,
            Err(Error::Type {
                type_error: TypeError {
                    expected_type: "usize",
                    storage_type: "Number"
                },
                value: String::from("string")
            })
        );
    }

    #[test]
    fn lower_modifier() {
        let input = Value::String(String::from("Hello World!"));
        let output = lower(&input, vec![]);

        assert_eq!(output, Ok(Value::String(String::from("hello world!"))));
    }

    #[test]
    fn upper_modifier() {
        let input = Value::String(String::from("Hello World!"));
        let output = upper(&input, vec![]);

        assert_eq!(output, Ok(Value::String(String::from("HELLO WORLD!"))));
    }
}
