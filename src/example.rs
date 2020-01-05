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
            // surname: Key<Surname>,
            // father: Option<Key<Person>>,
            // mother: Option<Key<Person>>,
            name: String,
        }
    }
}

struct Witness<F>(std::marker::PhantomData<F>);

impl<F> std::fmt::Debug for Witness<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str("Witness")
    }
}
impl<F> PartialEq for Witness<F> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

fn test<F: FnMut()>(mut f: F) -> Witness<F> {
    println!("running test");
    f();
    Witness(std::marker::PhantomData)
}

impl<F: 'static> Witness<F> {
    fn id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<F>()
    }
}

fn get_id() -> std::any::TypeId {
    let mut x = 5;
    test(move || { x += 1; }).id()
}

#[test]
fn testing() {
    assert_eq!(get_id(), get_id());
    // let w1 = test(|| ());
    // let w2 = test(|| ());
    // assert_eq!(w1.id(), w2.id());
    // assert_eq!(w1, w2);
    let f = || ();
    let w1 = test(f);
    let w2 = test(f);
    assert_eq!(w1, w2);
    assert_eq!(w1.id(), w2.id());
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
