use crate::fn_as_modifier;

use super::{ModifierGroup, AsModifier, ModifierCallback};

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
fn slice_modifier(input: String, start: usize, length: usize) -> String {
    let chars = input.chars().skip(start);
    chars.take(length).collect::<String>()
}

fn_as_modifier!(fn upper(input: &str) -> String => str::to_uppercase);

fn_as_modifier!(fn lower(input: &str) -> String => str::to_lowercase);

fn_as_modifier!(fn repeat(input: &str, n: usize) -> String => str::repeat);

pub struct StringModifierGroup;
impl ModifierGroup for StringModifierGroup {
    fn get_modifiers(&self) -> Vec<Box<dyn super::Modifier>> {
        let replace_modifier: &ModifierCallback = &replace_modifier;
        let slice_modifier: &ModifierCallback = &slice_modifier;
        let upper: &ModifierCallback = &upper;
        let lower: &ModifierCallback = &lower;
        let repeat: &ModifierCallback = &repeat;
        vec![
            Box::new(replace_modifier.as_modifier("replace")),
            Box::new(slice_modifier.as_modifier("slice")),
            Box::new(upper.as_modifier("upper")),
            Box::new(lower.as_modifier("lower")),
            Box::new(repeat.as_modifier("repeat"))
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::{value::{Value, TypeError}, modifier::{string::*, Error}};

    #[test]
    fn replace_modifier() {
        assert_eq!(
            super::replace_modifier(
                &Value::String("abcdefg".to_owned()),
                vec![
                    &Value::String("cde".to_owned()),
                    &Value::String("EDC".to_owned())
                ]
            ),
            Ok(Value::String("abEDCfg".to_owned()))
        );

        assert_eq!(
            super::replace_modifier(
                &Value::String("abcdefcdegcde".to_owned()),
                vec![
                    &Value::String("cde".to_owned()),
                    &Value::String("EDC".to_owned()),
                    &Value::Number(2usize.into())
                ]
            ),
            Ok(Value::String("abEDCfEDCgcde".to_owned()))
        );
    }

    #[test]
    fn slice_modifier() {
        let input = Value::String(String::from("Hello World!!!"));
        let start_in = Value::Number(6f64.into());
        let start_out = Value::Number(14f64.into());
        let length_5 = Value::Number(5f64.into());

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