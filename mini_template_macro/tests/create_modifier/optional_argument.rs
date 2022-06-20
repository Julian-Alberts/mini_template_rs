use mini_template_macro::create_modifier;

#[create_modifier]
fn modifier(s: String, r: usize, o: Option<bool>) -> String {
    format!("{} {} {:?}", s, r, o)
}

fn main() {
    let r = modifier(
        &mini_template::Value::String(String::from("FOO")),
        vec![
            &serde_json::json!(42),
            &mini_template::Value::Bool(true),
        ]
    );
    assert_eq!(r, Ok(mini_template::Value::String(String::from("FOO 42 Some(true)"))));

    let r = modifier(
        &mini_template::Value::String(String::from("FOO")),
        vec![
            &serde_json::json!(42),
        ]
    );
    assert_eq!(r, Ok(mini_template::Value::String(String::from("FOO 42 None"))));

    let r = modifier(
        &mini_template::Value::String(String::from("FOO")),
        vec![
            &serde_json::json!(42),
            &mini_template::Value::Null,
        ]
    );
    assert_eq!(r, Ok(mini_template::Value::String(String::from("FOO 42 None"))));
}