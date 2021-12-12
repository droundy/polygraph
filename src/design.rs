//! Ideas for what a generated API might look like.

polygraph!{
  struct Person {
     /// Biological father
     father: Option<Person>,
     /// Biological mother
     mother: Option<Person>,
     godparents: Set<Person>,
     surname: Surname,
     givenname: String,
  }

  struct Surname(String);
}

/// The above generates:
struct PersonDatum {
     /// Biological father
     father: Option<Key<Person>>,
     father_of: Set<Person>,
     /// Biological mother
     mother: Option<Key<Person>>,
     mother_of: Set<Person>,
     godparents: Set<Person>,
     godparent_of: Set<Person>,
     surname: Surname,
     givenname: String,
}

struct SurnameDatum {
  value: String,
  persons: Set<Person>,
}

struct Graphs {
  persons: Vec<PersonDatum>,
  surnames: Vec<SurnameDatum>,
}

pub struct Ref<'a, T> {
  graphs: &'a Graphs,
  key: Key<T>,
}
impl<'a> Ref<'a, Person> {
  fn father_of(& self) -> SetRef<Person> {}
}



