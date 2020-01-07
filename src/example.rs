//! # Example
//! ```
//! use polygraph::example::{Schema, Foo, Test};
//! let mut db = Schema::new(|| ());
//! let fortytwo = db.insert_foo(Foo(42));
//! assert_eq!(fortytwo.d(&db).0, 42); println!("about to add 50");
//! let fifty = db.insert_foo(Foo(50));
//! // We can't keep using the fifty and fortytwo above, because they mutably
//! // borrowed our `db`.  But we can now look them both up and use them both.
//! let fifty = db.lookup_foo(&Foo(50)).unwrap();
//! let fortytwo = db.lookup_foo(&Foo(42)).unwrap();
//! assert_eq!(fifty.d(&db).0 - fortytwo.d(&db).0, 8);
//! ```
//!
//! ```
//! let db = polygraph::example::tree::Tree::new(|| ());
//! ```

pub mod tree {
    polygraph_macro::schema!{
        type Tree;
        pub struct Surname(String);
        pub struct Person {
            surname: Key<Surname>,
            // father: Option<Key<Person>>,
            // mother: Option<Key<Person>>,
            name: String,
        }
    }

    #[test]
    fn test() {
        let mut db = Tree::new(|| ());
        let roundy = db.insert_surname(Surname("Roundy".to_string()));
        let _ = roundy.clone();
        let me = db.insert_person(Person {
            surname: roundy,
            name: "David".to_string()
        });
        me.clone();
    }
}

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
