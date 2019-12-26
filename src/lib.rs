#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub use polygraph_macro::schema;

pub mod example;

/// A key to an element in a table
pub struct Key<'a,T> {
    index: u32,
    phantom: std::marker::PhantomData<&'a T>,
}

pub struct SaveKey<T> {
    index: u32,
    phantom: std::marker::PhantomData<T>,
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
