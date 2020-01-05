polygraph::schema!{
    type Tree;
    pub struct Surname(String);
    pub struct Person {
        surname: Key<Surname<'a>>,
        name: String,
    }
}

fn main() {
}
