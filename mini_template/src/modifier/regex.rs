use std::sync::Arc;

use super::Modifier;

use {
    regex::Regex,
    std::{collections::HashMap, sync::RwLock},
};

#[derive(Default)]
pub struct RegexCache(RwLock<HashMap<String, Regex>>);
impl RegexCache {
    fn with_regex<F, T>(&self, regex_str: String, f: F) -> std::result::Result<T, String>
    where
        F: FnOnce(&Regex) -> T,
    {
        let cache_r = self.0.read().unwrap();
        let result = match cache_r.get(regex_str.as_str()) {
            Some(r) => (f)(r),
            None => {
                drop(cache_r);
                let regex = match Regex::new(&regex_str) {
                    Ok(r) => r,
                    Err(r) => return Err(r.to_string()),
                };
                let result = f(&regex);
                let mut cache_w = self.0.write().unwrap();
                cache_w.insert(regex_str, regex);
                result
            }
        };

        Ok(result)
    }
}

pub struct MatchModifier {
    cache: Arc<RegexCache>,
}
impl MatchModifier {
    #[mini_template_macro::create_modifier(modifier_ident = "callable")]
    fn match_modifier(
        &self,
        input: String,
        regex: String,
        group: Option<usize>,
    ) -> std::result::Result<String, String> {
        let group = group.unwrap_or(0);
        self.cache.with_regex(regex, |regex| {
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
}
impl Modifier for MatchModifier {
    fn name(&self) -> &str {
        "match"
    }
    fn call(
        &self,
        subject: &crate::value::Value,
        args: Vec<&crate::value::Value>,
    ) -> super::Result<crate::value::Value> {
        self.callable(subject, args)
    }
}

pub struct ReplaceRegexModifier {
    cache: Arc<RegexCache>,
}
impl ReplaceRegexModifier {
    #[mini_template_macro::create_modifier(modifier_ident = "callable")]
    fn replace_regex_modifier(
        &self,
        input: String,
        regex: String,
        to: String,
        count: Option<usize>,
    ) -> std::result::Result<String, String> {
        let count = count.unwrap_or(0);
        self.cache
            .with_regex(regex, |regex| regex.replacen(&input, count, to).to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        modifier::{
            regex::{MatchModifier, ReplaceRegexModifier},
            Error,
        },
        value::Value,
    };

    #[test]
    fn match_modifier() {
        let mm = MatchModifier {
            cache: Default::default(),
        };
        let input = Value::String(String::from("My 2test2 string"));
        let regex = Value::String(String::from(r#"(\d[a-z]+\d) string"#));
        let invalid_regex = Value::String(String::from(r#"(\d[a-z]+\d string"#));
        let not_matching_regex = Value::String(String::from("\\d{2}"));
        let not_matching_group_regex = Value::String(String::from(".(\\d{2})?"));
        let full_match = Value::Number(0usize.into());
        let group = Value::Number(1usize.into());
        let args = vec![&regex, &full_match];

        let result = mm.callable(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2 string"))));

        let args = vec![&regex];

        let result = mm.callable(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2 string"))));

        let args = vec![&regex, &group];
        let result = mm.callable(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2"))));

        let args = vec![&invalid_regex, &full_match];
        let result = mm.callable(&input, args);
        assert_eq!(
            result,
            Err(Error::Modifier(
                "regex parse error:\n    (\\d[a-z]+\\d string\n    ^\nerror: unclosed group"
                    .to_owned()
            ))
        );

        let args = vec![&not_matching_regex, &full_match];
        let result = mm.callable(&input, args);
        assert_eq!(result, Ok(Value::String(String::from(""))));

        let args = vec![&not_matching_group_regex, &group];
        let result = mm.callable(&input, args);
        assert_eq!(result, Ok(Value::String(String::from(""))));
    }

    #[test]
    fn replace_regex_modifier() {
        let input = Value::String(String::from("Hello World!!!"));
        let regex = Value::String("Wo(rld)".to_owned());
        let replacement = Value::String("FooBar".to_owned());
        let rpm = ReplaceRegexModifier {
            cache: Default::default(),
        };
        assert_eq!(
            rpm.callable(&input, vec![&regex, &replacement]),
            Ok(Value::String("Hello FooBar!!!".to_owned()))
        );
    }
}
