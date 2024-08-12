use crate::registry::Item::Pair;
use crate::registry::Item::Square;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

use crate::{book_helper::Book, item::Item, line::Line, ortho::Ortho};
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Registry {
    pub squares: HashSet<Ortho>,
    pub pairs: HashSet<Line>,
    pub name: String,
    pub provenance: Vec<String>,
}
impl Registry {
    #[allow(dead_code)]
    pub(crate) fn from_text(text: &str, filename: &str, number: usize) -> Self {
        Self::from_book(&Book::book_from_text(filename, text, number))
    }

    pub(crate) fn count_by_shape(&self) -> Vec<(Vec<usize>, usize)> {
        let mut coll: HashMap<Vec<usize>, usize> = HashMap::default();

        for o in self.squares.iter() {
            let count = coll.entry(o.shape.clone()).or_default();
            *count += 1;
        }

        coll.into_iter().collect_vec()
    }

    pub fn forward(&self, from: String) -> HashSet<String> {
        self.pairs
            .iter()
            .filter(|l| l.first == from)
            .map(|l| l.second.clone())
            .collect()
    }

    pub fn backward(&self, to: String) -> HashSet<String> {
        self.pairs
            .iter()
            .filter(|l| l.second == to)
            .map(|l| l.first.clone())
            .collect()
    }

    pub(crate) fn left_of(&self, item: &Item) -> Vec<Item> {
        // for line-line-line relationships, this is as simple as:
        // left: a-b
        // center: a-c
        // right: c-d
        // a-b
        // |
        // c-d

        // for square-line-square relationships, you get:
        // left: origin, hop, contents. Start with origin:
        // left square.origin = a
        // center: a-b
        // right: other_square.origin = b

        // if the item is a square, you have a square in the center which is not useful.
        // return an empty for now but consider unbundling item out and managing these calls separately
        // to avoid this situation
        match item {
            Item::Pair(l) => self
                .lines_starting_with(&l.first)
                .iter()
                .chain(self.squares_with_origin(&l.first).iter())
                .cloned()
                .collect(),
            Item::Square(_) => vec![],
        }
    }

    pub(crate) fn right_of(&self, item: &Item) -> Vec<Item> {
        match item {
            Item::Pair(l) => self
                .lines_starting_with(&l.second)
                .iter()
                .chain(self.squares_with_origin(&l.second).iter())
                .cloned()
                .collect(),
            Item::Square(_) => vec![],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn minus(&self, target_answer: &Self) -> Self {
        let self_prov: HashSet<_> = HashSet::from_iter(self.provenance.clone());
        let other_prov: HashSet<_> = HashSet::from_iter(target_answer.provenance.clone());
        let new_provenance: Vec<String> = self_prov.difference(&other_prov).cloned().collect_vec();
        Registry {
            squares: self
                .squares
                .difference(&target_answer.squares)
                .cloned()
                .collect(),
            pairs: self
                .pairs
                .difference(&target_answer.pairs)
                .cloned()
                .collect(),
            name: self.name.clone(),
            provenance: new_provenance,
        }
    }

    pub(crate) fn union(&self, target_answer: &Self) -> Self {
        let mut new_name = self.name.clone();
        new_name.push_str(&target_answer.name); // todo revert names to not be combined

        Registry {
            squares: self
                .squares
                .union(&target_answer.squares)
                .cloned()
                .collect(),
            pairs: self.pairs.union(&target_answer.pairs).cloned().collect(),
            name: new_name,
            provenance: self
                .provenance
                .iter()
                .chain(target_answer.provenance.iter())
                .cloned()
                .collect_vec(),
        }
    }

    pub(crate) fn add(&self, additional_squares: Vec<Ortho>) -> Self {
        Registry {
            squares: self
                .squares
                .union(&additional_squares.iter().cloned().collect())
                .cloned()
                .collect(),
            pairs: self.pairs.clone(),
            name: self.name.clone(),
            provenance: self.provenance.clone(),
        }
    }

    pub(crate) fn from_book(book: &Book) -> Self {
        Registry {
            squares: HashSet::default(),
            pairs: book.make_pairs(),
            name: book.calculate_name(),
            provenance: vec![book.calculate_name()],
        }
    }

    pub(crate) fn contains(&self, item: &Item) -> bool {
        match item {
            Pair(l) => self.pairs.contains(l),
            Square(s) => self.squares.contains(s),
        }
    }

    fn lines_starting_with(&self, first: &String) -> Vec<Item> {
        self.pairs
            .iter()
            .filter(|l| &l.first == first)
            .map(Pair)
            .collect_vec()
    }

    pub(crate) fn contains_line_with(&self, f: &str, s: &str) -> bool {
        self.pairs.contains(&Line {
            first: f.to_owned(),
            second: s.to_owned(),
        })
    }

    pub(crate) fn items(&self) -> Vec<Item> {
        self.squares
            .iter()
            .map(Square)
            .chain(self.pairs.iter().map(Pair))
            .collect()
    }

    pub fn squares_with_origin(&self, origin: &str) -> Vec<Item> {
        self.squares
            .iter()
            .filter(|o| o.origin() == origin)
            .map(Item::Square)
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use crate::{folder::single_process, registry::Registry};

    #[test]
    fn test_count_by_shape() {
        let r = Registry::from_text("a b c d. a c. b d.", "first.txt", 1);
        let res = single_process(&r);

        assert_eq!(res.count_by_shape(), vec![(vec![2, 2], 1)])
    }
}
