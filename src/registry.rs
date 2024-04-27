use itertools::Itertools;
use serde::{Deserialize, Serialize};
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
    pub(crate) fn size(&self) -> usize {
        self.squares.len() + self.pairs.len()
    }

    // todo later chain in squares to cover the domain
    // todo rename this or make it actually iter
    pub(crate) fn iter(&self) -> Vec<Item> {
        self.pairs
            .iter()
            .map(|l| Item::Line(l.clone()))
            .collect_vec()
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
        if let crate::registry::Item::Line(line) = item {
            self.lines_starting_with(&line.first)
        } else {
            todo!()
        }
    }

    pub(crate) fn right_of(&self, item: &Item) -> Vec<Item> {
        if let crate::registry::Item::Line(line) = item {
            self.lines_starting_with(&line.second)
        } else {
            todo!()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn minus(&self, target_answer: &Self) -> Self {
        // todo separate out the name holder from the rest. Names and provenance are only useful when writing out. The other fields are useful during run
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
            provenance: vec![self.name.clone()],
        }
    }

    pub(crate) fn union(&self, target_answer: &Self) -> Self {
        Registry {
            squares: self
                .squares
                .union(&target_answer.squares)
                .cloned()
                .collect(),
            pairs: self.pairs.union(&target_answer.pairs).cloned().collect(),
            name: self.name.clone(),
            provenance: vec![self.name.clone()],
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
            provenance: vec![self.name.clone()],
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
        if let crate::registry::Item::Line(line) = item {
            self.pairs.contains(line)
        } else {
            todo!()
        }
    }

    fn lines_starting_with(&self, first: &String) -> Vec<Item> {
        self.pairs
            .iter()
            .filter(|l| &l.first == first)
            .map(|l| Item::Line(l.clone()))
            .collect_vec()
    }
    
    pub(crate) fn contains_line_with(&self, f: &String, s: &String) -> bool {
        self.pairs.contains(&Line { first: f.clone(), second: s.clone() })
    }
}
