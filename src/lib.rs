#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub use polygraph_macro::schema;

pub mod example;

pub struct SchemaGen<T, F: Fn() -> std::rc::Rc<T>>( F );
pub struct InternalSchema<T, F: Fn() -> std::rc::Rc<T>>( std::marker::PhantomData<F> );
impl<T, F: Fn() -> std::rc::Rc<T>> SchemaGen<T,F> {
    fn internal(self) -> InternalSchema<T,F> {
        InternalSchema( std::marker::PhantomData )
    }
}
#[macro_export]
macro_rules! singleton {
    ($t: ty) => {{
        let internal_data = std::rc::Rc::new(<$t>::default());
        let f = move || { internal_data.clone() };
        SchemaGen(f).internal()
    }}
}

#[test]
fn test_singleton() {
    let x = singleton!(u32);
}

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
