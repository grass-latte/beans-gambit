use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

pub trait DenseIntegerMapKey: Clone + Copy {
    fn max_index() -> usize;
    fn from_index(index: usize) -> Self;
    fn as_index(&self) -> usize;
}

#[derive(Clone)]
pub struct DenseIntegerMap<Key, Value>
where
    Key: DenseIntegerMapKey,
    Value: Clone,
{
    table: Vec<Option<Value>>,
    phantom_key: PhantomData<Key>,
}

impl<Key, Value> DenseIntegerMap<Key, Value>
where
    Key: DenseIntegerMapKey,
    Value: Clone,
{
    pub fn new() -> Self {
        Self {
            table: vec![None; Key::max_index()],
            phantom_key: PhantomData,
        }
    }

    pub fn contains(&self, key: Key) -> bool {
        self.table[key.as_index()].is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Key, &Value)> {
        self.table
            .iter()
            .filter_map(Option::as_ref)
            .enumerate()
            .map(|(index, value)| (Key::from_index(index), value))
    }
}

impl<Key, Value> Index<Key> for DenseIntegerMap<Key, Value>
where
    Key: DenseIntegerMapKey,
    Value: Clone,
{
    type Output = Value;

    fn index(&self, key: Key) -> &Self::Output {
        self.table[key.as_index()].as_ref().expect("key not in map")
    }
}

impl<Key, Value> IndexMut<Key> for DenseIntegerMap<Key, Value>
where
    Key: DenseIntegerMapKey,
    Value: Clone,
{
    fn index_mut(&mut self, key: Key) -> &mut Self::Output {
        self.table[key.as_index()].as_mut().expect("key not in map")
    }
}
