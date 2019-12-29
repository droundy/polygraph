//! # Example
//! ```
//! use polygraph::example::{Schema, Foo, Test};
//! let mut db = Schema::<bool>::new().unwrap();
//! let fortytwo = db.insert_foo(Foo(42));
//! assert_eq!(fortytwo.0, 42);
//! let fifty = db.insert_foo(Foo(50));
//! // We can't keep using the fifty and fortytwo above, because they mutably
//! // borrowed our `db`.  But we can now look them both up and use them both.
//! let fifty = db.lookup_foo(&Foo(50)).unwrap();
//! let fortytwo = db.lookup_foo(&Foo(42)).unwrap();
//! assert_eq!(fifty.0 - fortytwo.0, 8);
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
