use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mini_template::{value::Value, MiniTemplate};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simple template", |b| b.iter(|| {
        let mut mini = MiniTemplate::default();
        mini.add_template(black_box("test_template"), black_box(String::from("Hello {var|upper}! \n {var2|repeat:10}")))
    })).bench_function("longer template", |b| b.iter(|| {
        let mut mini = MiniTemplate::default();
        mini.add_template(black_box("test_template"), black_box(String::from("Hello {var|upper}! \n {var2|repeat:10} long literal text {var3|modifier:123:\"5433\":true}")))
    })).bench_function("render simple template", |b| b.iter_with_setup(|| {
        let mut mini = MiniTemplate::default();
        mini.add_default_modifiers();
        mini.add_template(String::from("tpl"), String::from("Hello {var|upper}! \n {var2|lower}")).unwrap();

        let mut vars = HashMap::new();
        vars.insert("var".to_owned(), Value::String(String::from("world")));
        vars.insert("var2".to_owned(), Value::String(String::from("TEST STRING")));
        (mini, vars)
    }, |(mini, vars)| {
        mini.render(&String::from("tpl"), &vars).unwrap();
    })).final_summary();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
