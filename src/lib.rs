pub use polygraph_macro::schema;

pub mod example;

pub mod new;

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
