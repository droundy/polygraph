use std::marker::PhantomData;

use tinyset::SetUsize;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Key<T> {
    index: usize,
    phantom: PhantomData<T>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct KeySet<T> {
    set: SetUsize,
    phantom: PhantomData<Key<T>>,
}

impl<T> Default for KeySet<T> {
    fn default() -> Self {
        KeySet { set: SetUsize::new(), phantom: PhantomData }
    }
}

impl<T> KeySet<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, v: Key<T>) -> bool {
        self.set.insert(v.index)
    }
    pub fn contains(&self, v: Key<T>) -> bool {
        self.set.contains(v.index)
    }
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Key<T>> + 'a {
        self.set.iter().map(|index| Key {
            index,
            phantom: PhantomData,
        })
    }
}
