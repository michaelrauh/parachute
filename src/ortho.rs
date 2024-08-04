use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use memoize::memoize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
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
        index_array(&self.shape)
            .iter()
            .enumerate()
            .filter(|(_, x)| x.iter().sum::<usize>() == 1)
            .map(|(i, _)| self.contents[i].clone())
            .collect()
    }

    pub(crate) fn origin(&self) -> &String {
        &self.contents[0]
    }

    pub(crate) fn dimensionality(&self) -> usize {
        self.shape.len()
    }

    pub(crate) fn contents(&self) -> Vec<String> {
        self.contents
            .iter()
            .filter(|x| x != &self.origin() && !self.get_hop().contains(x))
            .cloned()
            .collect_vec()
    }

    pub(crate) fn connection_works(
        // todo test
        &self,
        other_connection: String,
        registry: &crate::registry::Registry,
        correspondence: &[(String, String)],
        r: &&Ortho,
    ) -> bool {
        todo!()
    }

    pub(crate) fn zip_up(&self, r: &Ortho, correspondence: &[(String, String)]) -> Ortho {
        let scrambled_right = self.apply_correspondence(correspondence, r);
        Ortho {
            contents: self
                .contents
                .iter()
                .cloned()
                .chain(scrambled_right)
                .collect(),
            shape: self.shape.iter().cloned().chain(vec![2]).collect(),
        }
    }

    fn apply_correspondence(&self, correspondence: &[(String, String)], r: &Ortho) -> Vec<String> {
        let moves: HashMap<usize, usize> = correspondence
            .iter()
            .map(|(left_corr, right_corr)| {
                let left_position = self.get_one_hot(left_corr);
                let right_position = r.get_one_hot(right_corr);
                (left_position, right_position)
            })
            .collect();
        r.scramble(moves)
    }

    fn get_one_hot(&self, left_corr: &String) -> usize {
        // get coord of the "one" coord of the location of the member
        let pos = self
            .contents
            .iter()
            .find_position(|x| x == &left_corr)
            .unwrap()
            .0;
        let coords = &index_array(&self.shape)[pos];
        coords.iter().find_position(|x| **x == 1).unwrap().0
    }

    fn scramble(&self, moves: HashMap<usize, usize>) -> Vec<String> {
        // look at the from/to mapping of positions and apply that to each position in the index array. Spit out members in order according to the index array.
        let all_coords = &index_array(&self.shape);
        let mut target = vec!["".to_owned(); self.contents.len()];
        for item in &self.contents {
            let pos = self
                .contents
                .iter()
                .find_position(|x| x == &item)
                .unwrap()
                .0;
            let coords = &all_coords[pos];
            let new_coords = map_coords(&moves, coords);
            let target_position = all_coords
                .iter()
                .find_position(|x| x == &&new_coords)
                .unwrap()
                .0;
            target[target_position] = item.to_owned();
        }
        target
    }

    fn zip_over(&self, r: &Ortho, corr: &[(String, String)], dir: String) -> Ortho {
        let new_r = self.apply_correspondence(corr, r);
        let dir_pos = self.get_one_hot(&dir);
        let positions_to_keep = self.get_indices_maxing_position(dir_pos);
        let mut elements_to_keep = positions_to_keep
            .iter()
            .map(|i| new_r[*i].clone())
            .rev()
            .collect_vec();
        let mut new_coords = self.shape.clone();
        new_coords[dir_pos] += 1;

        let target_indices = index_array(&new_coords);
        let left_indices = index_array(&self.shape);
        let mut target = vec!["".to_string(); target_indices.len()];
        for (pos, idx) in left_indices.iter().enumerate() {
            let target_index = target_indices
                .iter()
                .find_position(|x| x == &idx)
                .unwrap()
                .0;
            target[target_index] = self.contents[pos].clone();
        }
        for (index, item) in target.clone().iter().enumerate() {
            if item == &"".to_string() {
                target[index] = elements_to_keep.pop().unwrap()
            }
        }
        Ortho {
            contents: target,
            shape: new_coords,
        }
    }

    fn get_indices_maxing_position(&self, dir_pos: usize) -> Vec<usize> {
        let positions = index_array(&self.shape);

        // this could be done in one combined pass by tracking max, index, and positions concurrently.
        // it would be necessary to delete all positions found so far if a new max is identified.
        let mut max = 0;
        for pos in &positions {
            let cur = pos[dir_pos];
            if cur > max {
                max = cur;
            }
        }
        positions
            .iter()
            .enumerate()
            .filter(|(_, x)| x[dir_pos] == max)
            .map(|(i, _)| i)
            .collect()
    }
}

fn map_coords(moves: &HashMap<usize, usize>, coords: &[usize]) -> Vec<usize> {
    let mut target = vec![0; coords.len()];
    for (k, v) in moves {
        target[*v] = coords[*k];
    }
    target
}

#[memoize]
fn diagonal_template(shape: Vec<usize>) -> Vec<HashSet<usize>> {
    let index_array = index_array(&shape);
    let max_distance = index_array.last().unwrap().iter().sum::<usize>();
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
        let lhs_shape = self.shape.iter().collect::<HashSet<_>>();
        let rhs_shape = other.shape.iter().collect::<HashSet<_>>();
        if lhs_shape != rhs_shape {
            return false;
        }

        let template = diagonal_template(self.shape.clone());
        for template_bucket in template {
            let mut left_bucket = HashSet::new();
            let mut right_bucket = HashSet::new();

            for location in template_bucket {
                left_bucket.insert(self.contents[location].clone());
                right_bucket.insert(other.contents[location].clone());
            }

            if left_bucket != right_bucket {
                return false;
            }
        }
        true
    }
}

impl std::hash::Hash for Ortho {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let lhs_shape = self.shape.iter().collect::<HashSet<_>>();
        let template = diagonal_template(self.shape.clone());
        for template_bucket in template.clone() {
            let mut left_bucket = HashSet::new();

            for location in template_bucket {
                left_bucket.insert(self.contents[location].clone());
            }
        }
        for bucket in template {
            let mut sorted = bucket.iter().collect_vec();
            sorted.sort();

            sorted.hash(state);
        }

        let mut sorted = lhs_shape.iter().collect_vec();
        sorted.sort();

        sorted.hash(state);
    }
}

#[cfg(test)]
mod tests {

    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::ortho::{index_array, Ortho};

    #[test]
    fn test_hop() {
        let o = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        assert_eq!(vec!["b".to_string(), "c".to_string()], o.get_hop())
    }

    #[test]
    fn test_zip_up() {
        // a b  +  e f
        // c d     g h

        // ==
        // a b
        // c d

        // e g
        // f h

        let l = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        let r = Ortho::new(
            "e".to_string(),
            "f".to_string(),
            "g".to_string(),
            "h".to_string(),
        );
        let corr = vec![
            ("b".to_string(), "g".to_string()),
            ("c".to_string(), "f".to_string()),
        ];
        let res = l.zip_up(&r, &corr);

        assert_eq!(res.origin(), &"a".to_string());
        assert_eq!(
            res.get_hop(),
            vec!["b".to_string(), "c".to_string(), "e".to_string()]
        );
        assert_eq!(res.dimensionality(), 3);
        assert_eq!(res.shape, vec![2, 2, 2]);
        assert_eq!(
            res.contents(),
            vec![
                "d".to_string(),
                "g".to_string(),
                "f".to_string(),
                "h".to_string()
            ]
        );
    }

    #[test]
    fn test_zip_over() {
        // a b  +  b d   a b e
        // c d     e f   c d f
        // combine mapping b=e, c=d along b axis
        // [a   b  c d] + [b   e  d  f] = [a  b   e  c  d  f]
        // [00 01 10 11]  [00 01 10 11]   [00 01 02 10 11 12]
        // algorithm:
        // LHS stays put in terms of its index array
        // RHS:
        // - scramble the RHS to be in the same coordinate system as the LHS using the mapping like you're going to zip up. Use dims of LHS for RHS once this is done.
        // - find out which coordinate position the dir points to. Find the elements that max that value in the RHS (in this case, second position is b and max value is 1)
        // - bump the coordinate from the direction. Get one hot on the target on LHS and bump that index in dims from LHS
        // Finally - create an empty with the target dims. Insert LHS and RHS. Make sure dims are in the right order and one is bumped.

        let l = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        let r = Ortho::new(
            "b".to_string(),
            "d".to_string(),
            "e".to_string(),
            "f".to_string(),
        );
        let corr = vec![
            ("b".to_string(), "e".to_string()),
            ("c".to_string(), "d".to_string()),
        ];
        let dir = "b".to_string();

        let res = l.zip_over(&r, &corr, dir);
        assert_eq!(res.origin(), &"a".to_string());
        assert_eq!(res.get_hop(), vec!["b".to_string(), "c".to_string()]);
        assert_eq!(res.dimensionality(), 2);
        assert_eq!(res.shape, vec![2, 3]);
        assert_eq!(
            res.contents(),
            vec!["e".to_string(), "d".to_string(), "f".to_string()]
        );
    }

    #[test]
    fn things_are_not_equal_if_they_are_different_sizes_even_if_packed_data_is_the_same_length() {
        // a b : (2, 2, 2) !=  i j : (2, 4)
        // c d                 k l
        //                     m n
        // e f                 o p
        // g h
        let abcd = Ortho::new(
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        );
        let efgh = Ortho::new(
            "e".to_owned(),
            "f".to_owned(),
            "g".to_owned(),
            "h".to_owned(),
        );

        let lhs = abcd.zip_up(
            &efgh,
            &[
                ("b".to_owned(), "f".to_owned()),
                ("c".to_owned(), "g".to_owned()),
            ],
        );

        let ijkl = Ortho::new(
            "i".to_owned(),
            "j".to_owned(),
            "k".to_owned(),
            "l".to_owned(),
        );
        let klmn = Ortho::new(
            "k".to_owned(),
            "l".to_owned(),
            "m".to_owned(),
            "n".to_owned(),
        );
        let mnop = Ortho::new(
            "m".to_owned(),
            "n".to_owned(),
            "o".to_owned(),
            "p".to_owned(),
        );

        // i j
        // k l
        //
        // k l
        // m n
        let ijklmn = ijkl.zip_over(
            &klmn,
            &[
                ("k".to_owned(), "m".to_owned()),
                ("j".to_owned(), "l".to_owned()),
            ],
            "k".to_owned(),
        );

        // k l
        // m n
        //
        // m n
        // o p

        // i j
        // k l
        // m n

        // k l
        // m n
        // o p
        let klmnop = klmn.zip_over(
            &mnop,
            &[
                ("m".to_owned(), "o".to_owned()),
                ("l".to_owned(), "n".to_owned()),
            ],
            "m".to_owned(),
        );
        let rhs = ijklmn.zip_over(
            &klmnop,
            &[
                ("k".to_owned(), "m".to_owned()),
                ("j".to_owned(), "l".to_owned()),
            ],
            "k".to_owned(),
        );

        assert_ne!(lhs, rhs)
    }

    #[test]
    fn things_can_be_equal_even_if_the_buckets_are_scrambled() {
        let abcd = Ortho::new(
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        );
        let acbd = Ortho::new(
            "a".to_owned(),
            "c".to_owned(),
            "b".to_owned(),
            "d".to_owned(),
        );

        assert_eq!(abcd, acbd)
    }

    #[test]
    fn things_can_be_unequal_when_they_are_different() {
        let abcd = Ortho::new(
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        );
        let gbcd = Ortho::new(
            "g".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        );
        let agcd = Ortho::new(
            "a".to_owned(),
            "g".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        );
        let abgd = Ortho::new(
            "a".to_owned(),
            "b".to_owned(),
            "g".to_owned(),
            "d".to_owned(),
        );
        let abcg = Ortho::new(
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "g".to_owned(),
        );

        assert_eq!(abcd, abcd);
        assert_ne!(abcd, gbcd);
        assert_ne!(abcd, agcd);
        assert_ne!(abcd, abgd);
        assert_ne!(abcd, abcg);
    }

    #[test]
    fn hash_is_rotation_independent() {
        let abcd = Ortho::new(
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        );
        let acbd = Ortho::new(
            "a".to_owned(),
            "c".to_owned(),
            "b".to_owned(),
            "d".to_owned(),
        );

        let mut lhs_hasher = DefaultHasher::new();
        abcd.hash(&mut lhs_hasher);
        let mut rhs_hasher = DefaultHasher::new();
        acbd.hash(&mut rhs_hasher);
        assert_eq!(lhs_hasher.finish(), rhs_hasher.finish());
    }

    #[test]
    fn hashes_detect_differences() {
        let abcd = Ortho::new(
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        );
        let gbcd = Ortho::new(
            "g".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        );

        let mut lhs_hasher = DefaultHasher::new();
        abcd.hash(&mut lhs_hasher);
        let mut rhs_hasher = DefaultHasher::new();
        gbcd.hash(&mut rhs_hasher);
        assert_eq!(lhs_hasher.finish(), rhs_hasher.finish());
    }
}
