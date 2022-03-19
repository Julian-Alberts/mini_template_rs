use mini_template_macro::create_modifier;

#[create_modifier]
fn modifier(s: String, r: usize, o: Option<bool>) -> String {
    format!("{} {} {:?}", s, r, o)
}

fn main() {
    let r = modifier(
        &mini_template::value::Value::String(String::from("FOO")),
        vec![
            &mini_template::value::Value::Number(42.),
            &mini_template::value::Value::Bool(true),
        ]
    );
    assert_eq!(r, Ok(mini_template::value::Value::String(String::from("FOO 42 Some(true)"))));

    let r = modifier(
        &mini_template::value::Value::String(String::from("FOO")),
        vec![
            &mini_template::value::Value::Number(42.),
        ]
    );
    assert_eq!(r, Ok(mini_template::value::Value::String(String::from("FOO 42 None"))));

    let r = modifier(
        &mini_template::value::Value::String(String::from("FOO")),
        vec![
            &mini_template::value::Value::Number(42.),
            &mini_template::value::Value::Null,
        ]
    );
    assert_eq!(r, Ok(mini_template::value::Value::String(String::from("FOO 42 None"))));
}