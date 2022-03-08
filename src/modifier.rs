use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::Hasher, sync::RwLock};

use once_cell::sync::OnceCell;
use regex::Regex;

use super::value::Value;
pub use error::*;

static REGEX_CACHE: OnceCell<RwLock<HashMap<u64, Regex>>> = OnceCell::new();

#[macro_export]
macro_rules! create_modifier {
    (fn $modifier_name: ident ($first_name:ident: $first_t: ty $($(,$name: ident: $t: ty $(= $default: expr)?)+)?) -> Result<$return: ty> $b: block) => {
        #[allow(unused_variables)]
        pub fn $modifier_name(value: &$crate::value::Value, args: Vec<&$crate::value::Value>) -> $crate::modifier::error::Result<$crate::value::Value> {
            use $crate::modifier::error::Error;
            use std::convert::TryInto;

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

            fn inner($first_name: $first_t $($(,$name: $t)+)?) -> std::result::Result<$return, String> $b

            let result = inner($first_name $($(,$name)+)?).or_else(|e| Err(Error::Modifier(e)))?;
            Ok(result.into())
        }
    };
    (fn $modifier_name: ident ($first_name:ident: $first_t: ty $($(,$name: ident: $t: ty $(= $default: expr)?)+)?) -> $return: ty $b: block) => {
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

            fn inner($first_name: $first_t $($(,$name: $t)+)?) -> $return $b

            let result = inner($first_name $($(,$name)+)?);
            Ok(result.into())
        }
    };
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

create_modifier!(
    fn slice_modifier(input: String, start: usize, length: usize) -> String {
        let chars = input.chars().skip(start);
        chars.take(length).collect::<String>()
    }
);

create_modifier!(fn match_modifier(input: String, regex: String, group: usize = 0) -> Result<String> {
    with_regex_from_cache(regex, |regex| {
        match regex.captures(&input[..]) {
            Some(c) => match c.get(group) {
                Some(c) => c.as_str(),
                None => ""
            },
            None => ""
        }.to_owned()
    })
});

create_modifier!(fn replace_modifier(input: String, from: String, to: String, count: usize = 0) -> String {
    if count == 0 {
        input.replace(&from[..], &to[..])
    } else {
        input.replacen(&from[..], &to[..], count)
    }
});

create_modifier!(fn replace_regex_modifier(input: String, regex: String, to: String, count: usize = 0) -> Result<String> {
    with_regex_from_cache(regex, |regex| {
        regex.replacen(&input, count, to).to_string()
    })
});

create_modifier!(fn upper(input: &str) -> String => str::to_uppercase);

create_modifier!(fn lower(input: &str) -> String => str::to_lowercase);

create_modifier!(
    fn add(a: f64, b: f64) -> f64 {
        a + b
    }
);

create_modifier!(
    fn sub(a: f64, b: f64) -> f64 {
        a - b
    }
);

create_modifier!(
    fn mul(a: f64, b: f64) -> f64 {
        a * b
    }
);

create_modifier!(
    fn div(a: f64, b: f64) -> f64 {
        a / b
    }
);

create_modifier!(fn repeat(input: &str, n: usize) -> String => str::repeat);

fn with_regex_from_cache<F, T>(regex: String, f: F) -> std::result::Result<T, String> 
    where F: FnOnce(&Regex) -> T
{
    use std::hash::Hash;
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
                Err(r) => return Err(r.to_string())
            };
            let result = f(&regex);
            let mut cache_w = cache.write().unwrap();
            cache_w.insert(cache_key, regex);
            result
        }
    };

    Ok(result)
}

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

        let result = super::match_modifier(&input, args);
        assert_eq!(
            result,
            Err(Error::MissingArgument {
                argument_name: "regex"
            })
        );
    }

    #[test]
    fn can_not_parse_argument() {
        let input = Value::String(String::from("My test string"));

        let regex = Value::String(String::from(r#"(\d[a-z]+\d) string"#));
        let number = Value::String(String::from("test"));

        let args = vec![&regex, &number];

        let result = super::match_modifier(&input, args);
        assert_eq!(
            result,
            Err(Error::Type {
                type_error: TypeError {
                    expected_type: "usize",
                    storage_type: "Number"
                },
                value: String::from("test")
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
