use mini_template_macro::create_modifier;

#[create_modifier]
fn modifier(s: String) -> Result<String, String> {
    if s == "FOO" {
        Ok(s)
    } else {
        Err(s)
    }
}

fn main() {
    let r = modifier(&mini_template::value::Value::String(String::from("FOO")), vec![]);
    assert_eq!(r, Ok(mini_template::value::Value::String(String::from("FOO"))));

    let r = modifier(&mini_template::value::Value::String(String::from("BAR")), vec![]);
    assert_eq!(r, Err(mini_template::modifier::Error::Modifier(String::from("BAR"))))
}