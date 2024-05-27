use std::{collections::HashSet, hash::Hash};

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

    pub(crate) fn origin(&self) -> &String {
        todo!()
    }
    
    pub(crate) fn get_hop(&self) -> Vec<String> {
        todo!()
    }
    
    pub(crate) fn dimensionality(&self) -> usize {
        todo!()
    }
    
    pub(crate) fn zip_up(&self, r: &&Ortho, correspondence: &Vec<(String, String)>) -> Vec<Ortho> {
        todo!()
    }
    
    pub(crate) fn contents(&self) -> Vec<String> {
        todo!()
    }
    
    pub(crate) fn connection_works(&self, other_connection: String, registry: &crate::registry::Registry, correspondence: &[(String, String)], r: &&Ortho) -> bool {
        todo!()
    }
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
    fn eq(&self, other: &Self) -> bool {
        if self.shape != other.shape {
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