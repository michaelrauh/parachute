use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

use crate::bag::Bag;
use crate::{book_helper::Book, line::Line, ortho::Ortho};
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Registry {
    pub squares: HashSet<Ortho>,
    pub pairs: HashSet<Line>,
    pub name: String,
    pub provenance: Vec<String>,
    pub squares_by_origin: HashMap<String, Vec<Ortho>>,
}
impl Registry {
    #[allow(dead_code)]
    pub fn from_text(text: &str, filename: &str, number: usize) -> Self {
        Self::from_book(&Book::book_from_text(filename, text, number))
    }

    pub(crate) fn count_by_shape(&self) -> Vec<(Bag<usize>, usize)> {
        let mut coll: HashMap<Bag<usize>, usize> = HashMap::default();

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

    pub(crate) fn line_left_of(&self, item: &Line) -> Vec<Line> {
        // for line-line-line relationships, this is as simple as:
        // left: a-b
        // center: a-c
        // right: c-d
        // a-b
        // |
        // c-d

        self.lines_starting_with(&item.first)
            .iter()
            .cloned()
            .collect()
    }

    pub(crate) fn square_left_of<'a>(&'a self, item: &Line) -> impl Iterator<Item = &'a Ortho> + 'a {
        // for square-line-square relationships, you get:
        // left: origin, hop, contents. Start with origin:
        // left square.origin = a
        // center: a-b
        // right: other_square.origin = b
        self.squares_with_origin(&item.first)
    }

    pub(crate) fn line_right_of(&self, item: &Line) -> Vec<Line> {
        self.lines_starting_with(&item.second)
            .iter()
            .cloned()
            .collect()
    }

    pub(crate) fn ortho_right_of<'a>(&'a self, item: &Line) -> impl Iterator<Item = &'a Ortho>  + 'a {
        self.squares_with_origin(&item.second)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn minus(&self, target_answer: &Self) -> Self {
        let self_prov: HashSet<_> = HashSet::from_iter(self.provenance.clone());
        let other_prov: HashSet<_> = HashSet::from_iter(target_answer.provenance.clone());
        let new_provenance: Vec<String> = self_prov.difference(&other_prov).cloned().collect_vec();
        let all_squares: HashSet<Ortho> = self
            .squares
            .difference(&target_answer.squares)
            .cloned()
            .collect();

        let new_sbo: HashMap<String, Vec<Ortho>> = all_squares
            .iter()
            .cloned()
            .into_group_map_by(|o| o.origin().to_string());

        Registry {
            squares: all_squares,
            pairs: self
                .pairs
                .difference(&target_answer.pairs)
                .cloned()
                .collect(),
            name: self.name.clone(),
            provenance: new_provenance,
            squares_by_origin: new_sbo,
        }
    }

    pub(crate) fn union(&self, target_answer: &Self) -> Self {
        let all_squares: HashSet<Ortho> = self
            .squares
            .union(&target_answer.squares)
            .cloned()
            .collect();
        let new_sbo: HashMap<String, Vec<Ortho>> = all_squares
            .iter()
            .cloned()
            .into_group_map_by(|o| o.origin().to_string());

        Registry {
            squares: all_squares,
            pairs: self.pairs.union(&target_answer.pairs).cloned().collect(),
            name: self.name.clone(),
            provenance: self
                .provenance
                .iter()
                .chain(target_answer.provenance.iter())
                .cloned()
                .collect_vec(),
            squares_by_origin: new_sbo,
        }
    }

    pub(crate) fn add(&self, additional_squares: Vec<Ortho>) -> Self {
        let all_squares: HashSet<_> = self
            .squares
            .union(&additional_squares.iter().cloned().collect())
            .cloned()
            .collect();
        let new_sbo: HashMap<String, Vec<Ortho>> = all_squares
            .iter()
            .cloned()
            .into_group_map_by(|o| o.origin().to_string());
        Registry {
            squares: all_squares,
            pairs: self.pairs.clone(),
            name: self.name.clone(),
            provenance: self.provenance.clone(),
            squares_by_origin: new_sbo,
        }
    }

    pub(crate) fn from_book(book: &Book) -> Self {
        Registry {
            squares: HashSet::default(),
            pairs: book.make_pairs(),
            name: book.calculate_name(),
            provenance: vec![book.calculate_name()],
            squares_by_origin: HashMap::default(),
        }
    }

    pub(crate) fn contains_line(&self, item: &Line) -> bool {
        self.pairs.contains(item)
    }

    pub(crate) fn contains_ortho(&self, item: &Ortho) -> bool {
        self.squares.contains(item)
    }

    fn lines_starting_with(&self, first: &String) -> Vec<Line> {
        self.pairs
            .iter()
            .filter(|l| &l.first == first)
            .cloned()
            .collect_vec()
    }

    pub(crate) fn contains_line_with(&self, f: &str, s: &str) -> bool {
        self.pairs.contains(&Line {
            first: f.to_owned(),
            second: s.to_owned(),
        })
    }

    pub(crate) fn lines(&self) -> Vec<Line> {
        self.pairs.iter().cloned().collect()
    }

    pub(crate) fn orthos(&self) -> Vec<Ortho> {
        self.squares.iter().cloned().collect()
    }

    pub fn squares_with_origin<'a>(&'a self, origin: &str) -> impl Iterator<Item = &'a Ortho> + 'a {
        self.squares_by_origin
            .get(origin)
            .into_iter()
            .flat_map(|v| v.iter())
    }    
}

#[cfg(test)]
mod tests {

    use crate::{bag::Bag, folder::single_process, registry::Registry};

    #[test]
    fn test_count_by_shape() {
        let r = Registry::from_text("a b c d. a c. b d.", "first.txt", 1);
        let res = single_process(&r);

        assert_eq!(res.count_by_shape(), vec![(Bag::from_iter(vec![2, 2]), 1)])
    }
}
