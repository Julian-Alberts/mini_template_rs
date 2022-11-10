use {
    once_cell::sync::OnceCell,
    regex::Regex,
    std::{
        collections::{hash_map::DefaultHasher, HashMap},
        hash::Hash,
        sync::RwLock,
    },
};

static REGEX_CACHE: OnceCell<RwLock<HashMap<u64, Regex>>> = OnceCell::new();

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

#[mini_template_macro::create_modifier]
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
fn replace_regex_modifier(
    input: String,
    regex: String,
    to: String,
    count: Option<usize>,
) -> std::result::Result<String, String> {
    let count = count.unwrap_or(0);
    with_regex_from_cache(regex, |regex| regex.replacen(&input, count, to).to_string())
}

pub fn regex_cache_clear() {
    REGEX_CACHE.set(RwLock::default()).unwrap();
}

#[cfg(test)]
mod tests {
    use crate::{value::Value, modifier::Error};

    #[test]
    fn match_modifier() {
        let input = Value::String(String::from("My 2test2 string"));
        let regex = Value::String(String::from(r#"(\d[a-z]+\d) string"#));
        let invalid_regex = Value::String(String::from(r#"(\d[a-z]+\d string"#));
        let not_matching_regex = Value::String(String::from("\\d{2}"));
        let not_matching_group_regex = Value::String(String::from(".(\\d{2})?"));
        let full_match = Value::Number(0usize.into());
        let group = Value::Number(1usize.into());
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
    fn replace_regex_modifier() {
        let input = Value::String(String::from("Hello World!!!"));
        let regex = Value::String("Wo(rld)".to_owned());
        let replacement = Value::String("FooBar".to_owned());
        assert_eq!(
            super::replace_regex_modifier(&input, vec![&regex, &replacement]),
            Ok(Value::String("Hello FooBar!!!".to_owned()))
        );
    }
}
