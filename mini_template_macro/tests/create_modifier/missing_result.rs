use mini_template_macro::create_modifier;

#[create_modifier]
fn modifier(s: String) {
    s
}

fn main() {}