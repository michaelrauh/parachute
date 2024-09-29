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
    pub forward_pairs: HashMap<String, HashSet<String>>,
    pub backward_pairs: HashMap<String, HashSet<String>>,
    empty_set: HashSet<String>,
    lines_by_start: HashMap<String, HashSet<Line>>,
}

impl Registry {
    #[allow(dead_code)]
    pub fn from_text(text: &str, filename: &str, number: usize) -> Self {
        Self::from_book(&Book::book_from_text(filename, text, number))
    }

    pub(crate) fn count_by_shape<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'a Bag<usize>, usize)> + 'a {
        let mut coll: HashMap<&'a Bag<usize>, usize> = HashMap::new();

        for o in &self.squares {
            let count = coll.entry(&o.shape).or_insert(0);
            *count += 1;
        }

        coll.into_iter()
    }

    pub fn forward(&self, from: &str) -> &HashSet<String> {
        self.forward_pairs.get(from).unwrap_or(&self.empty_set)
    }

    pub fn backward(&self, to: &str) -> &HashSet<String> {
        self.backward_pairs.get(to).unwrap_or(&self.empty_set)
    }

    pub(crate) fn line_left_of<'a>(&'a self, item: &'a Line) -> impl Iterator<Item = &'a Line> {
        self.lines_starting_with(&item.first)
    }

    pub(crate) fn square_left_of<'a>(
        &'a self,
        item: &Line,
    ) -> impl Iterator<Item = &'a Ortho> + 'a {
        self.squares_with_origin(&item.first)
    }

    pub(crate) fn line_right_of<'a>(&'a self, item: &'a Line) -> impl Iterator<Item = &'a Line> {
        self.lines_starting_with(&item.second)
    }

    pub(crate) fn ortho_right_of<'a>(
        &'a self,
        item: &Line,
    ) -> impl Iterator<Item = &'a Ortho> + 'a {
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

        let new_pairs: HashSet<Line> = self
            .pairs
            .difference(&target_answer.pairs)
            .cloned()
            .collect();
        Registry {
            squares: all_squares,
            pairs: new_pairs.clone(),
            name: self.name.clone(),
            provenance: new_provenance,
            squares_by_origin: new_sbo,
            forward_pairs: make_pairs_to_forward(&new_pairs),
            backward_pairs: make_pairs_to_backward(&new_pairs),
            empty_set: HashSet::default(),
            lines_by_start: make_lines_by_start(&new_pairs),
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

        let new_pairs: HashSet<Line> = self.pairs.union(&target_answer.pairs).cloned().collect();
        Registry {
            squares: all_squares,
            pairs: new_pairs.clone(),
            name: self.name.clone(),
            provenance: self
                .provenance
                .iter()
                .chain(target_answer.provenance.iter())
                .cloned()
                .collect_vec(),
            squares_by_origin: new_sbo,
            forward_pairs: make_pairs_to_forward(&new_pairs),
            backward_pairs: make_pairs_to_backward(&new_pairs),
            empty_set: HashSet::default(),
            lines_by_start: make_lines_by_start(&new_pairs),
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
            forward_pairs: make_pairs_to_forward(&self.pairs),
            backward_pairs: make_pairs_to_backward(&self.pairs),
            empty_set: HashSet::default(),
            lines_by_start: make_lines_by_start(&self.pairs),
        }
    }

    pub(crate) fn from_book(book: &Book) -> Self {
        let pairs = book.make_pairs();
        Registry {
            squares: HashSet::default(),
            pairs: pairs.clone(),
            name: book.calculate_name(),
            provenance: vec![book.calculate_name()],
            squares_by_origin: HashMap::default(),
            forward_pairs: make_pairs_to_forward(&pairs),
            backward_pairs: make_pairs_to_backward(&pairs),
            empty_set: HashSet::default(),
            lines_by_start: make_lines_by_start(&pairs),
        }
    }

    pub(crate) fn contains_line(&self, item: &Line) -> bool {
        self.pairs.contains(item)
    }

    pub(crate) fn contains_ortho(&self, item: &Ortho) -> bool {
        self.squares.contains(item)
    }

    fn lines_starting_with<'a>(&'a self, first: &'a String) -> impl Iterator<Item = &'a Line> + 'a {
        self.lines_by_start
            .get(first)
            .into_iter()
            .flat_map(|v| v.iter())
    }

    pub(crate) fn contains_line_with(&self, f: &str, s: &str) -> bool {
        self.pairs.contains(&Line {
            first: f.to_owned(),
            second: s.to_owned(),
        })
    }

    pub(crate) fn lines(&self) -> impl Iterator<Item = &Line> {
        self.pairs.iter()
    }

    pub(crate) fn orthos(&self) -> impl Iterator<Item = &Ortho> {
        self.squares.iter()
    }

    pub fn squares_with_origin<'a>(&'a self, origin: &str) -> impl Iterator<Item = &'a Ortho> + 'a {
        self.squares_by_origin
            .get(origin)
            .into_iter()
            .flat_map(|v| v.iter())
    }
}

pub fn make_pairs_to_backward(new_pairs: &HashSet<Line>) -> HashMap<String, HashSet<String>> {
    let mut backward_pairs = HashMap::new();
    for pair in new_pairs {
        backward_pairs
            .entry(pair.second.clone())
            .or_insert_with(HashSet::new)
            .insert(pair.first.clone());
    }
    backward_pairs
}

pub fn make_pairs_to_forward(pairs: &HashSet<Line>) -> HashMap<String, HashSet<String>> {
    let mut forward_pairs = HashMap::new();
    for pair in pairs {
        forward_pairs
            .entry(pair.first.clone())
            .or_insert_with(HashSet::new)
            .insert(pair.second.clone());
    }
    forward_pairs
}

pub fn make_lines_by_start(pairs: &HashSet<Line>) -> HashMap<String, HashSet<Line>> {
    let mut lines_by_start = HashMap::new();
    for pair in pairs {
        lines_by_start
            .entry(pair.first.clone())
            .or_insert_with(HashSet::new)
            .insert(pair.clone());
    }
    lines_by_start
}

#[cfg(test)]
mod tests {

    use crate::{bag::Bag, folder::single_process, registry::Registry};

    #[test]
    fn test_count_by_shape() {
        use std::iter::FromIterator;

        let r = Registry::from_text("a b c d. a c. b d.", "first.txt", 1);
        let res = single_process(&r);

        let mut count_by_shape: Vec<(Bag<usize>, usize)> = res
            .count_by_shape()
            .map(|(bag, count)| (bag.clone(), count))
            .collect();

        count_by_shape.sort_by(|a, b| a.0.cmp(&b.0));

        let expected = vec![(Bag::from_iter(vec![2, 2]), 1)];

        assert_eq!(count_by_shape, expected);
    }
}
