use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mini_template::{value::Value, value_iter, MiniTemplateBuilder, ValueManager};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simple template", |b| b.iter(|| {
        let mut mini = MiniTemplateBuilder::default().build();
        mini.add_template(black_box("test_template".to_owned()), black_box(String::from("Hello {var|upper}! \n {var2|repeat:10}")))
    })).bench_function("longer template", |b| b.iter(|| {
        let mut mini = MiniTemplateBuilder::default().build();
        mini.add_template(black_box("test_template".to_owned()), black_box(String::from("Hello {var|upper}! \n {var2|repeat:10} long literal text {var3|modifier:123:\"5433\":true}")))
    })).bench_function("render simple template", |b| b.iter_with_setup(|| {
        let mut mini = MiniTemplateBuilder::default()
        .with_default_modifiers().build();
        mini.add_template(String::from("tpl"), String::from("Hello {var|upper}! \n {var2|lower}")).unwrap();

        let vars = ValueManager::try_from_iter(value_iter!(
            "var": Value::String(String::from("world")),
            "var2": Value::String(String::from("TEST STRING"))
        )).unwrap();
        (mini, vars)
    }, |(mini, vars)| {
        mini.render(&String::from("tpl"), vars).unwrap();
    })).final_summary();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
