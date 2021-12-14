//! Ideas for what a generated API might look like.

#[drive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Key<T> {
  index: usize,
  phantom: PhantomData<T>,
}

#[drive(Clone, PartialEq, Eq, Hash)]
pub struct KeySet<T> {
  set: SetUsize,
  phantom: PhantomData<Key<T>>,
}

polygraph!{
  struct Person {
     /// Biological father
     father: Option<Key<Person>>,
     /// Biological mother
     mother: Option<Key<Person>>,
     godparents: Set<Person>,
     surname: Surname,
     givenname: String,
  }

  struct Surname(String);
}

/// The above generates:
struct Person {
     /// Biological father
     father: Option<Key<Person>>,
     #[serde(ignore)]
     father_of: Set<Person>,
     /// Biological mother
     mother: Option<Key<Person>>,
     #[serde(ignore)]
     mother_of: Set<Person>,
     godparents: Set<Person>,
     #[serde(ignore)]
     godparent_of: Set<Person>,
     surname: Surname,
     givenname: String,
}

struct Surname {
  value: String,
  #[serde(ignore)]
  persons: Set<Person>,
}

#[drive(Clone, PartialEq, Eq, Hash, Serializes, Deserialize)]
pub struct Graphs {
  person: Vec<PersonDatum>,
  surname: Vec<SurnameDatum>,
}

#[drive(Clone, PartialEq, Eq, Hash)]
pub struct RefSet<'a, T> {
  graphs: &'a Graphs,
  set: &'a KeySet<T>,
}
#[drive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Ref<'a, T> {
  graphs: &'a Graphs,
  key: Key<T>,
}
impl<'a> Deref Ref<'a, Person> {
  type Target = Person;
  fn deref(& self) -> & Person {
    &self.graphs.person[self.key.index]
  }
}
impl<'a> Ref<'a, Person> {
  fn father_of(&self) -> SetRef<Person> {
    RefSet {
      graphs: self.graphs,
      set: &self.graphs.person[self.key.index].father_of,
    }
  }
  fn father(&self) -> Option<Ref<Person>> {
    self.graphs.person[self.key.index].father.map(|key|
    Ref {
      graphs: self.graphs,
      key,
    })
  }
}



