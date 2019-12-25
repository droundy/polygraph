polygraph_macro::schema!{
    /// This is a cool test!
    pub struct Test(String);

    /// This is another cool test named Foo
    pub struct Foo(u64);
}

/// This is not in the schema.
pub enum NotHere {
    A, B,
}
