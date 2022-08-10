#[cfg(feature = "regex")]
use {
    once_cell::sync::OnceCell,
    regex::Regex,
    std::{
        collections::{hash_map::DefaultHasher, HashMap},
        hash::Hash,
        sync::RwLock,
    },
};

use super::value::Value;
use crate::{fn_as_modifier, ValueManager};
use core::ops::{Add, Div, Mul, Sub};
pub use error::*;

#[cfg(feature = "regex")]
static REGEX_CACHE: OnceCell<RwLock<HashMap<u64, Regex>>> = OnceCell::new();

pub type Modifier = dyn Fn(&Value, Vec<&Value>) -> Result<Value>;

#[mini_template_macro::create_modifier]
fn slice_modifier(input: String, start: usize, length: usize) -> String {
    let chars = input.chars().skip(start);
    chars.take(length).collect::<String>()
}

#[cfg(feature = "regex")]
#[mini_template_macro::create_modifier(returns_result = true)]
fn match_modifier(
    input: String,
    regex: String,
    group: Option<usize>,
) -> std::result::Result<String, String> {
    let group = group.unwrap_or(0);
    with_regex_from_cache(regex, |regex| {
        match regex.captures(&input[..]) {
            Some(c) => match c.get(group) {
                Some(c) => c.as_str(),
                None => "",
            },
            None => "",
        }
        .to_owned()
    })
}

#[mini_template_macro::create_modifier]
fn replace_modifier(input: String, from: String, to: String, count: Option<usize>) -> String {
    let count = count.unwrap_or(0);
    if count == 0 {
        input.replace(&from[..], &to[..])
    } else {
        input.replacen(&from[..], &to[..], count)
    }
}

#[mini_template_macro::create_modifier]
fn len_modifier(vm: &ValueManager) -> usize {
    vm.len()
}

#[cfg(feature = "regex")]
#[mini_template_macro::create_modifier(returns_result = true)]
fn replace_regex_modifier(
    input: String,
    regex: String,
    to: String,
    count: Option<usize>,
) -> std::result::Result<String, String> {
    let count = count.unwrap_or(0);
    with_regex_from_cache(regex, |regex| regex.replacen(&input, count, to).to_string())
}

fn_as_modifier!(fn upper(input: &str) -> String => str::to_uppercase);

fn_as_modifier!(fn lower(input: &str) -> String => str::to_lowercase);

fn_as_modifier!(fn add(a: f64, b: f64) -> f64 => f64::add);

fn_as_modifier!(fn sub(a: f64, b: f64) -> f64 => f64::sub);

fn_as_modifier!(fn mul(a: f64, b: f64) -> f64 => f64::mul);

fn_as_modifier!(fn div(a: f64, b: f64) -> f64 => f64::div);

fn_as_modifier!(fn repeat(input: &str, n: usize) -> String => str::repeat);

#[cfg(feature = "regex")]
fn with_regex_from_cache<F, T>(regex: String, f: F) -> std::result::Result<T, String>
where
    F: FnOnce(&Regex) -> T,
{
    use std::hash::Hasher;
    let mut hasher = DefaultHasher::new();
    regex.hash(&mut hasher);
    let cache_key = hasher.finish();

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
                Self::Modifier(e) => write!(f, "{e}"),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{value::TypeError, value_iter};

    use super::*;

    #[cfg(feature = "regex")]
    #[test]
    fn match_modifier() {
        let input = Value::String(String::from("My 2test2 string"));
        let regex = Value::String(String::from(r#"(\d[a-z]+\d) string"#));
        let invalid_regex = Value::String(String::from(r#"(\d[a-z]+\d string"#));
        let not_matching_regex = Value::String(String::from("\\d{2}"));
        let not_matching_group_regex = Value::String(String::from(".(\\d{2})?"));
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
        );

        let args = vec![&not_matching_regex, &full_match];
        let result = super::match_modifier(&input, args);
        assert_eq!(result, Ok(Value::String(String::from(""))));

        let args = vec![&not_matching_group_regex, &group];
        let result = super::match_modifier(&input, args);
        assert_eq!(result, Ok(Value::String(String::from(""))));
    }

    #[test]
    fn replace_modifier() {
        assert_eq!(super::replace_modifier(&Value::String("abcdefg".to_owned()), vec![
            &Value::String("cde".to_owned()),
            &Value::String("EDC".to_owned())
        ]), Ok(Value::String("abEDCfg".to_owned())));

        assert_eq!(super::replace_modifier(&Value::String("abcdefcdegcde".to_owned()), vec![
            &Value::String("cde".to_owned()),
            &Value::String("EDC".to_owned()),
            &Value::Number(2.)
        ]), Ok(Value::String("abEDCfEDCgcde".to_owned())));
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
    fn replace_regex_modifier() {
        let input = Value::String(String::from("Hello World!!!"));
        let regex = Value::String("Wo(rld)".to_owned());
        let replacement = Value::String("FooBar".to_owned());
        assert_eq!(
            super::replace_regex_modifier(&input, vec![&regex, &replacement]),
            Ok(Value::String("Hello FooBar!!!".to_owned()))
        );
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

    #[test]
    fn len() {
        let object = Value::Object(ValueManager::default());
        assert_eq!(len_modifier(&object, vec![]), Ok(Value::Number(0.)));

        let object = Value::Object(ValueManager::try_from_iter(value_iter!(
            "a": Value::Null,
            "b": Value::Null
        )).unwrap());
        assert_eq!(len_modifier(&object, vec![]), Ok(Value::Number(2.)));
    }
}
