#[test]
fn create_modifier() {
    let t = trybuild::TestCases::new();
    t.pass("tests/create_modifier/value_only.rs");
    t.pass("tests/create_modifier/attributs.rs");
    t.pass("tests/create_modifier/return_result.rs");
    t.pass("tests/create_modifier/optional_argument.rs");
    t.compile_fail("tests/create_modifier/argument_is_optional_and_default.rs");
    t.compile_fail("tests/create_modifier/missing_value_argument.rs");
    t.compile_fail("tests/create_modifier/missing_result.rs");
    t.pass("tests/create_modifier/optional_value_argument.rs");
    t.pass("tests/create_modifier/unused_arg.rs");
}
