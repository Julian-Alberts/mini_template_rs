use mini_template_macro::create_modifier;

#[create_modifier]
fn modifier(value: String, _: usize) -> String {
    value
}

fn main() {}