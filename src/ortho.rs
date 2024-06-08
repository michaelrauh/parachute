use std::{collections::{HashMap, HashSet}, hash::Hash};

use itertools::Itertools;
use memoize::memoize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq)]
pub struct Ortho {
    contents: Vec<String>,
    shape: Vec<usize>,
}

impl Ortho {
    pub fn new(a: String, b: String, c: String, d: String) -> Self {
        Ortho {
            contents: vec![a, b, c, d],
            shape: vec![2, 2],
        }
    }


    pub fn get_hop(&self) -> Vec<String> {
        Self::get_positions_to_visit(&self.shape)
            .into_iter()
            .map(|position| &self.contents[position])
            .cloned()
            .collect()
    }

    // #[memoize]
    fn get_positions_to_visit(dims: &Vec<usize>) -> Vec<usize> {
        let to_visit: Vec<usize> = dims.iter().rev().skip(1).cloned().collect();
        (0..=to_visit.len())
            .map(|i| Self::mult_up_to(i, &to_visit))
            .collect()
    }

    fn mult_up_to(position: usize, l: &[usize]) -> usize {
        l.iter().take(position).product()
    }


    pub(crate) fn origin(&self) -> &String {
        &self.contents[0]
    }
    
    pub(crate) fn dimensionality(&self) -> usize {
        self.shape.len()
    }

    pub(crate) fn contents(&self) -> Vec<String> {
        self.contents.iter().filter(|x| x != &self.origin() && !self.get_hop().contains(x)).cloned().collect_vec()
    }

    pub(crate) fn connection_works(&self, other_connection: String, registry: &crate::registry::Registry, correspondence: &[(String, String)], r: &&Ortho) -> bool {
        todo!()
    }
    
    pub(crate) fn zip_up(&self, r: &Ortho, correspondence: &Vec<(String, String)>) -> Ortho {
        let moves: HashMap<usize, usize> = correspondence.into_iter().map(|(left_corr, right_corr)| {
            let left_position = self.get_one_hot(left_corr);
            let right_position = r.get_one_hot(right_corr);
            (left_position, right_position)
        }).collect();
        let scrambled_right = r.scramble(moves);
        Ortho {
            contents: self.contents.iter().cloned().chain(scrambled_right).collect(),
            shape: self.shape.iter().cloned().chain(vec![2]).collect(),
        }
    }
    
    fn get_one_hot(&self, left_corr: &String) -> usize {
        // get coord of the "one" coord of the location of the member
        let pos = self.contents.iter().find_position(|x| {x == &left_corr}).unwrap().0;
        let coords = &index_array(&self.shape)[pos];
        coords.iter().find_position(|x| **x == 1).unwrap().0
    }
    
    fn scramble(&self, moves: HashMap<usize, usize>) -> Vec<String> {
        // look at the from/to mapping of positions and apply that to each position in the index array. Spit out members in order according to the index array.
        let all_coords = &index_array(&self.shape);
        let mut target = vec!["".to_owned(); self.contents.len()];
        for item in &self.contents {
            let pos = self.contents.iter().find_position(|x| {x == &item}).unwrap().0;
            let coords = &all_coords[pos];
            let new_coords = map_coords(&moves, coords);
            let target_position = all_coords.iter().find_position(|x| x == &&new_coords).unwrap().0;
            target[target_position] = item.to_owned();
        }
        target
    }
}

fn map_coords(moves: &HashMap<usize, usize>, coords: &[usize]) -> Vec<usize> {
    let mut target = vec![0;coords.len()];
    for (k, v) in moves {
        target[*v] = coords[*k];
    }
    target
}

#[memoize]
fn diagonal_template(shape: Vec<usize>) -> Vec<HashSet<usize>> {
    let index_array = index_array(&shape);
    let max_distance = index_array
        .last()
        .unwrap()
        .iter()
        .fold(1, |total, current| total * current);
    let mut ans = vec![HashSet::new(); max_distance + 1];

    for (i, index) in index_array.iter().enumerate() {
        let distance: usize = index.iter().sum();
        ans[distance].insert(i);
    }

    ans
}

fn index_array(dims: &[usize]) -> Vec<Vec<usize>> {
    cartesian_product(dims.iter().map(|x| (0..*x).collect()).collect())
}

fn partial_cartesian<T: Clone>(a: Vec<Vec<T>>, b: Vec<T>) -> Vec<Vec<T>> {
    a.into_iter()
        .flat_map(|xs| {
            b.iter()
                .cloned()
                .map(|y| {
                    let mut vec = xs.clone();
                    vec.push(y);
                    vec
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn cartesian_product<T: Clone>(lists: Vec<Vec<T>>) -> Vec<Vec<T>> {
    match lists.split_first() {
        Some((first, rest)) => {
            let init: Vec<Vec<T>> = first.iter().cloned().map(|n| vec![n]).collect();

            rest.iter()
                .cloned()
                .fold(init, |vec, list| partial_cartesian(vec, list))
        }
        None => {
            vec![]
        }
    }
}

impl PartialEq for Ortho {
    fn eq(&self, other: &Self) -> bool { // todo test
        if self.shape != other.shape { // todo fix. Shape may be scrambled.
            return false;
        }

        let template = diagonal_template(self.shape.clone());
        for template_bucket in template {
            let mut left_bucket = HashSet::new();
            let mut right_bucket = HashSet::new();

            for location in template_bucket {
                left_bucket.insert(self.contents[location].clone());
                right_bucket.insert(self.contents[location].clone());
            }

            if left_bucket != right_bucket {
                return false;
            }
        }
        return true;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ortho::Ortho;


    #[test]
    fn test_hop() {
        let o = Ortho::new("a".to_string(), "b".to_string(), "c".to_string(), "d".to_string());
        assert_eq!(vec!["b".to_string(), "c".to_string()], o.get_hop())
    }

    #[test]
    fn test_zip_up() {
        let l = Ortho::new("a".to_string(), "b".to_string(), "c".to_string(), "d".to_string());
        let r = Ortho::new("e".to_string(), "f".to_string(), "g".to_string(), "h".to_string());
        let corr = vec![("b".to_string(), "g".to_string()), ("c".to_string(), "f".to_string())];
        let res = l.zip_up(&r, &corr);

        assert_eq!(res.origin(), &"a".to_string());
        assert_eq!(res.get_hop(), vec!["b".to_string(), "c".to_string(), "e".to_string()]);
        assert_eq!(res.dimensionality(), 3);
        assert_eq!(res.shape, vec![2,2,2]);
        assert_eq!(res.contents(), vec!["d".to_string(), "g".to_string(), "f".to_string(), "h".to_string()]);
    }

    #[test]
    fn things_are_not_equal_if_they_are_different_sizes_even_if_packed_data_is_the_same_length() {
        todo!()
    }

    #[test]
    fn things_can_be_equal_even_if_the_buckets_are_scrambled() {
        todo!()
    }

    #[test]
    fn things_can_be_unequal_when_they_are_different() {
        todo!()
    }
}