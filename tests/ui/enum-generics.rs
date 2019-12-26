polygraph::schema!{
    type Database;
    pub enum MyEnum<T> {
        Hello(T),
        Goodbye,
    }
}

fn main() {
}
