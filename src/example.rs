//! # Example
//! ```
//! use polygraph::example::{Schema, Foo, Test};
//! let mut db = Schema::<bool>::new().unwrap();
//! let fortytwo = db.insert_foo(Foo(42));
//! assert_eq!(fortytwo.0, 42);
//! let fifty = db.insert_foo(Foo(50));
//! ```

polygraph_macro::schema!{
    type Schema;
    /// This is a cool test!
    pub struct Test {
        pub name: String,
    }

    /// This is another cool test named Foo
    pub struct Foo(pub u64);
}

/// This is not in the schema.
pub enum NotHere {
    A, B,
}
