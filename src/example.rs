//! # Example
//! ```
//! use polygraph::example::{Schema, Foo, Test};
//! let mut db = Schema::new();
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
//! let db = polygraph::example::tree::Tree::new();
//! ```

pub mod tree {
    polygraph_macro::schema! {
        type Tree;
        /// By defining a `Surname` type, we can query which `Person` have a
        /// given surname.
        pub struct Surname(pub String);
        pub struct Person {
            pub last_name: Key<Surname>,
            /// `father` is a one-to-many relationship
            pub father: Option<Key<Person>>,
            /// `mother` is a one-to-many relationship
            pub mother: Option<Key<Person>>,
            /// We use a raw String for given name, since we don't care to
            /// iterate over all "Davids".  Otherwise we'd do the same as with [`Surname`].
            pub name: String,
            /// `dog` is a many-to-many relationship
            pub dog: KeySet<Dog>,
        }
        pub struct Dog {
            pub name: String,
        }
    }

    #[test]
    fn test() {
        let mut db = Tree::new();

        let mickey = db.insert_dog(Dog {
            name: "Mickey".to_string(),
        });
        let minnie = db.insert_dog(Dog {
            name: "Minnie".to_string(),
        });
        let my_dogs: tinyset::Set64<_> = [mickey, minnie].iter().cloned().collect();

        let roundy = db.insert_surname(Surname("Roundy".to_string()));
        let maiden_name = db.insert_surname(Surname("Maiden".to_string()));
        let me = db.insert_person(Person {
            last_name: roundy,
            father: None,
            mother: None,
            name: "David".to_string(),
            dog: my_dogs.clone(),
        });
        let wife = db.insert_person(Person {
            last_name: maiden_name,
            father: None,
            mother: None,
            name: "Monica".to_string(),
            dog: my_dogs.clone(),
        });
        let kid = db.insert_person(Person {
            last_name: roundy,
            father: Some(me),
            mother: Some(wife),
            name: "Kid".to_string(),
            dog: my_dogs.clone(),
        });
        assert_eq!(me.d(&db).last_name.d(&db).0, "Roundy");
        assert_eq!(wife.d(&db).last_name.d(&db).0, "Maiden");

        assert!(roundy.d(&db).last_name_of.contains(me));
        assert!(roundy.d(&db).last_name_of.contains(kid));
        assert!(me.d(&db).father_of.contains(kid));
        assert!(!me.d(&db).father_of.contains(wife));
        assert!(wife.d(&db).mother_of.contains(kid));

        assert!(minnie.d(&db).dog_of.contains(me));
        assert!(minnie.d(&db).dog_of.contains(wife));
        assert!(minnie.d(&db).dog_of.contains(kid));

        // assert!(me.d(&db).owner_of.contains(mickey));
        // assert!(me.d(&db).owner_of.contains(minnie));

        assert_eq!(db[db[me].last_name].0, "Roundy");
        assert_eq!(db[db[wife].last_name].0, "Maiden");

        assert!(db[roundy].last_name_of.contains(me));
        assert!(db[roundy].last_name_of.contains(kid));
        assert!(db[me].father_of.contains(kid));
        assert!(!db[me].father_of.contains(wife));

        // db.set_person(wife, Person {
        //     surname: roundy,
        //     name: "Monica".to_string()
        // });
        // assert_eq!(wife.d(&db).surname.d(&db).0, "Roundy");
    }
}

polygraph_macro::schema! {
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
    A,
    B,
}
