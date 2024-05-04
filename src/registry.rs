use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::registry::Item::Square;
use crate::registry::Item::Pair;



use crate::{book_helper::Book, item::Item, line::Line, ortho::Ortho};
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Registry {
    pub squares: HashSet<Ortho>,
    pub pairs: HashSet<Line>,
    pub name: String,
    pub provenance: Vec<String>,
}
impl Registry {
    pub(crate) fn number_of_pairs(&self) -> usize {
        self.pairs.len()
    }

    pub(crate) fn number_of_squares(&self) -> usize {
        self.squares.len()
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
        match item {
            Item::Pair(l) => self.lines_starting_with(&l.first),
            Item::Square(_) => todo!(),
        }
        
    }

    pub(crate) fn right_of(&self, item: &Item) -> Vec<Item> {
        match item {
            Pair(l) => self.lines_starting_with(&l.second),
            Square(_) => todo!(),
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
        Registry {
            squares: self
                .squares
                .union(&target_answer.squares)
                .cloned()
                .collect(),
            pairs: self.pairs.union(&target_answer.pairs).cloned().collect(),
            name: self.name.clone(),
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
            Square(_) => todo!(),
        }
    }

    fn lines_starting_with(&self, first: &String) -> Vec<Item> {
        self.pairs
            .iter()
            .filter(|l| &l.first == first)
            .map(|l| Pair(l))
            .collect_vec()
    }

    pub(crate) fn contains_line_with(&self, f: &String, s: &String) -> bool {
        self.pairs.contains(&Line {
            first: f.clone(),
            second: s.clone(),
        })
    }
    
    pub(crate) fn items(&self) -> Vec<Item> {
        self.squares.iter().map(|s| Square(s)).chain(self.pairs.iter().map(|p| Pair(&p))).collect()
    }
}
