use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, iter::FromIterator};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct Bag<T: Eq + Ord> {
    map: BTreeMap<T, usize>,
}

impl<T: Eq + Ord> Bag<T> {
    pub fn new() -> Self {
        Bag {
            map: BTreeMap::new(),
        }
    }

    pub fn add(mut self, item: T) -> Self {
        *self.map.entry(item).or_insert(0) += 1;
        self
    }

    pub fn remove(mut self, item: T) -> Self {
        if let Some(count) = self.map.get_mut(&item) {
            if *count > 1 {
                *count -= 1;
            } else {
                self.map.remove(&item);
            }
        }
        self
    }

    pub fn count(&self, item: &T) -> usize {
        *self.map.get(item).unwrap_or(&0)
    }

    pub fn len(&self) -> usize {
        self.map.values().sum()
    }

    pub fn unique_len(&self) -> usize {
        self.map.len()
    }

    pub fn contains(&self, item: &T) -> bool {
        self.map.contains_key(item)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.map
            .iter()
            .flat_map(|(item, count)| std::iter::repeat(item).take(*count))
    }
}

impl<T: Eq + Ord> FromIterator<T> for Bag<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        iter.into_iter().fold(Bag::new(), |bag, item| bag.add(item))
    }
}

impl<T: Eq + Ord> FromIterator<(T, usize)> for Bag<T> {
    fn from_iter<I: IntoIterator<Item = (T, usize)>>(iter: I) -> Self {
        let mut map = BTreeMap::new();
        for (item, count) in iter {
            map.insert(item, count);
        }
        Bag { map }
    }
}

impl<T: Eq + Ord> IntoIterator for Bag<T> {
    type Item = (T, usize);
    type IntoIter = std::collections::btree_map::IntoIter<T, usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter() // todo fix
    }
}