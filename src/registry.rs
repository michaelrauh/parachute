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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn add(&mut self, additional_squares: Vec<Ortho>) -> Vec<Ortho> {
        let mut added_squares = Vec::new();
        for ortho in additional_squares {
            if self.squares.insert(ortho.clone()) {
                let ortho_ref = self.squares.get(&ortho).unwrap();
                added_squares.push(ortho_ref.clone());
                self.squares_by_origin
                    .entry(ortho_ref.origin().to_string())
                    .or_insert_with(Vec::new)
                    .push(ortho_ref.clone());
            }
        }
        added_squares
    }

    pub(crate) fn add_lines(&mut self, lines: Vec<Line>) {
        for line in lines {
            self.pairs.insert(line.clone());
            self.forward_pairs
                .entry(line.first.clone())
                .or_insert_with(HashSet::new)
                .insert(line.second.clone());
            self.backward_pairs
                .entry(line.second.clone())
                .or_insert_with(HashSet::new)
                .insert(line.first.clone());
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
        }
    }

    pub(crate) fn contains_line_with(&self, f: &str, s: &str) -> bool {
        self.pairs.contains(&Line {
            first: f.to_owned(),
            second: s.to_owned(),
        })
    }

    pub fn squares_with_origin<'a>(&'a self, origin: &str) -> impl Iterator<Item = &'a Ortho> + 'a {
        self.squares_by_origin
            .get(origin)
            .into_iter()
            .flat_map(|v| v.iter())
    }

    pub(crate) fn subtract_lines<'a>(&'a self, target_answer: &'a Registry) -> HashSet<&'a Line> {
        self.pairs.difference(&target_answer.pairs).collect()
    }

    pub(crate) fn subtract_orthos<'a>(&'a self, target_answer: &'a Registry) -> HashSet<&'a Ortho> {
        self.squares.difference(&target_answer.squares).collect()
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

        let mut r = Registry::from_text("a b c d. a c. b d.", "first.txt", 1);
        single_process(&mut r);

        let mut count_by_shape: Vec<(Bag<usize>, usize)> = r
            .count_by_shape()
            .map(|(bag, count)| (bag.clone(), count))
            .collect();

        count_by_shape.sort_by(|a, b| a.0.cmp(&b.0));

        let expected = vec![(Bag::from_iter(vec![2, 2]), 1)];

        assert_eq!(count_by_shape, expected);
    }
}
