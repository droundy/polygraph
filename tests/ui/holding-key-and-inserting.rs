polygraph::schema!{
    type Database;
    pub struct Int(pub usize);
}

fn main() {
    let mut db = Database::<bool>::new().unwrap();
    let fortytwo = db.insert_int(Int(42));
    assert_eq!(fortytwo.0, 42);
    let fifty = db.insert_int(Int(50));
    assert_eq!(fortytwo.0, 42);
    assert_eq!(fifty.0, 50);
}
