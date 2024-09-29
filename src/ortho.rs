use std::{
    collections::{BTreeMap, BTreeSet},
    iter,
};

use serde::{Deserialize, Serialize};

use crate::bag::Bag;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ortho {
    shells: Vec<BTreeSet<String>>,
    pub shape: Bag<usize>,
    location_to_word: BTreeMap<Bag<String>, String>,
    word_to_location: BTreeMap<String, Bag<String>>,
}

impl Ortho {
    pub fn new(a: String, b: String, c: String, d: String) -> Self {
        let bag_a = Bag::new();
        let bag_b = Bag::from_iter(iter::once(b.clone()));
        let bag_c = Bag::from_iter(iter::once(c.clone()));
        let bag_d = Bag::from_iter(iter::once(b.clone()).chain(iter::once(c.clone())));
        Ortho {
            shells: vec![
                BTreeSet::from_iter(iter::once(a.clone())),
                BTreeSet::from_iter(iter::once(b.clone()).chain(iter::once(c.clone()))),
                BTreeSet::from_iter(iter::once(d.clone())),
            ],
            shape: Bag::from_iter(iter::once(2).chain(iter::once(2))),
            location_to_word: BTreeMap::from_iter(vec![
                (bag_a.clone(), a.clone()),
                (bag_b.clone(), b.clone()),
                (bag_c.clone(), c.clone()),
                (bag_d.clone(), d.clone()),
            ]),
            word_to_location: BTreeMap::from_iter(vec![
                (a, bag_a),
                (b, bag_b),
                (c, bag_c),
                (d, bag_d),
            ]),
        }
    }

    pub fn get_hop<'a>(&'a self) -> impl Iterator<Item = &'a String> + 'a {
        self.shells[1].iter()
    }

    pub(crate) fn origin(&self) -> &String {
        self.shells[0].iter().next().unwrap()
    }

    pub(crate) fn dimensionality(&self) -> usize {
        self.shape.len()
    }

    pub(crate) fn contents<'a>(&'a self) -> impl Iterator<Item = &'a String> + 'a {
        self.shells
            .iter()
            .skip(2)
            .flat_map(|set| set.iter())
    }

    pub(crate) fn connection_works(
        &self,
        self_word: String,
        registry: &crate::registry::Registry,
        correspondence: &[(String, String)],
        other_ortho: &Ortho,
    ) -> bool {
        let correspondence_map: BTreeMap<String, String> = correspondence.iter().cloned().collect();
        let corresponding_word = self.get_corresponding_word(
            &correspondence_map,
            other_ortho,
            &self_word,
        );
        registry.forward(&self_word).contains(&corresponding_word)
    }

    fn reverse(btree: BTreeMap<Bag<String>, String>) -> BTreeMap<String, Bag<String>> {
        let mut reversed = BTreeMap::new();
        for (bag, string_value) in btree {
            reversed.insert(string_value, bag);
        }
        reversed
    }

    pub(crate) fn zip_up(&self, r: &Ortho, correspondence: &[(String, String)]) -> Ortho {
        let correspondence_map: BTreeMap<String, String> = correspondence.iter().cloned().collect();
        let forward = self.zip_up_map(r, &correspondence_map);
        Ortho {
            shells: combine_shells(&self.shells, &r.shells),
            shape: self.shape.iter().cloned().chain(vec![2]).collect(),
            location_to_word: forward.clone(),
            word_to_location: Self::reverse(forward),
        }
    }

    pub fn zip_up_map(
        &self,
        r: &Ortho,
        old_axis_to_new_axis: &BTreeMap<String, String>,
    ) -> BTreeMap<Bag<String>, String> {
        let shift_axis = r.origin();
        let right_with_lefts_coordinate_system: BTreeMap<Bag<String>, String> = r
            .location_to_word
            .iter()
            .map(|(k, v)| {
                (
                    map_location(
                        &old_axis_to_new_axis
                            .iter()
                            .map(|(key, value)| (value.clone(), key.clone()))
                            .collect(),
                        k,
                    ),
                    v.clone(),
                )
            })
            .collect();
        let shifted_right: BTreeMap<Bag<String>, String> = right_with_lefts_coordinate_system
            .iter()
            .map(|(k, v)| (shift_location(shift_axis, k), v.clone()))
            .collect();
        let combined: BTreeMap<Bag<String>, String> = self
            .location_to_word
            .clone()
            .into_iter()
            .chain(shifted_right)
            .collect();
        combined
    }

    pub fn zip_over_map(
        &self,
        r: &Ortho,
        mapping: &BTreeMap<String, String>,
        shift_axis: String,
    ) -> BTreeMap<Bag<String>, String> {
        let new_shift_axis = self.get_corresponding_word(mapping, r, &shift_axis);
        let right_column = get_end(r, new_shift_axis.clone());
        let shifted: BTreeMap<Bag<String>, String> = right_column
            .into_iter()
            .map(|(k, v)| (k.add(new_shift_axis.clone()), v))
            .collect();
        let mapping_reversed: BTreeMap<String, String> = mapping
            .iter()
            .map(|(key, value)| (value.clone(), key.clone()))
            .collect();
        let mapped = shifted.into_iter().map(|(k, v)| {
            (
                map_location(
                    &mapping_reversed,
                    &k,
                ),
                v,
            )
        });
        let combined: BTreeMap<Bag<String>, String> = self
            .location_to_word
            .clone()
            .into_iter()
            .chain(mapped)
            .collect();
        combined
    }

    #[allow(dead_code)]
    fn zip_over(&self, r: &Ortho, corr: &[(String, String)], dir: String) -> Ortho {
        let correspondence_map: BTreeMap<String, String> = corr.iter().cloned().collect();
        let forward = self.zip_over_map(r, &correspondence_map, dir.clone());
        Ortho {
            shells: combine_shells(&self.shells, &r.shells),
            shape: self.calculate_shape(&dir),
            location_to_word: forward.clone(),
            word_to_location: Self::reverse(forward),
        }
    }

    pub(crate) fn valid_diagonal_with(&self, r: &Ortho) -> bool {
        let left_buckets = &self.shells[1..];
        let right_buckets = &r.shells[..r.shells.len() - 1];

        for (left_bucket, right_bucket) in left_buckets.iter().zip(right_buckets) {
            if !left_bucket.is_disjoint(right_bucket) {
                return false;
            }
        }

        true
    }

    fn get_corresponding_word(
        &self,
        correspondence: &BTreeMap<String, String>,
        other_ortho: &Ortho,
        self_word: &String,
    ) -> String {
        let location = &self.word_to_location[self_word];
        let target_location = map_location(&correspondence, &location);

        other_ortho.location_to_word[&target_location].clone()
    }

    fn calculate_shape(&self, dir: &str) -> Bag<usize> {
        let length = self.axis_length(dir.to_string()) + 1;
        self.shape.clone().add(length + 1).remove(length)
    }

    pub fn axis_length(&self, name: String) -> usize {
        self.location_to_word
            .keys()
            .max_by(|left, right| left.count(&name).cmp(&right.count(&name)))
            .expect("no empty orthos")
            .count(&name)
    }
}

fn get_end(r: &Ortho, shift_axis: String) -> BTreeMap<Bag<String>, String> {
    let axis_length = r.axis_length(shift_axis.clone());
    r.location_to_word
        .clone()
        .into_iter()
        .filter(|(k, _v)| k.count(&shift_axis) == axis_length)
        .collect()
}

fn combine_shells(l: &Vec<BTreeSet<String>>, r: &Vec<BTreeSet<String>>) -> Vec<BTreeSet<String>> {
    let empty_set = BTreeSet::new();

    l.iter()
        .zip(std::iter::once(&empty_set).chain(r.iter()))
        .map(|(left_bucket, right_bucket)| left_bucket.union(right_bucket).cloned().collect())
        .chain(std::iter::once(r.last().unwrap().clone()))
        .collect()
}

fn shift_location(shift_axis: &str, k: &Bag<String>) -> Bag<String> {
    k.to_owned().add(shift_axis.to_string())
}

fn map_location(old_axis_to_new_axis: &BTreeMap<String, String>, k: &Bag<String>) -> Bag<String> {
    k.to_owned()
        .into_iter()
        .map(|(strm, count)| (old_axis_to_new_axis[&strm].clone(), count))
        .collect()
}

#[cfg(test)]
mod tests {

    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use itertools::Itertools;

    use crate::{bag::Bag, ortho::Ortho};

    #[test]
    fn test_hop() {
        let o = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        let hop: Vec<String> = o.get_hop().cloned().collect();
        assert_eq!(vec!["b".to_string(), "c".to_string()], hop);
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
        let hop: Vec<String> = res.get_hop().cloned().collect();
        assert_eq!(
            hop,
            vec!["b".to_string(), "c".to_string(), "e".to_string()]
        );
        assert_eq!(res.dimensionality(), 3);
        assert_eq!(res.shape, Bag::from_iter(vec![2, 2, 2]));
        let contents: Vec<String> = res.contents().cloned().collect();
        assert_eq!(
            contents,
            vec![
                "d".to_string(),
                "f".to_string(),
                "g".to_string(),
                "h".to_string()
            ]
        );
    }

    #[test]
    fn test_zip_over() {
        // a b  +  b d   a b e
        // c d     e f   c d f
        // combine mapping b=e, c=d along b axis
        // 0 1 2
        // 1 2 3
        // [ {a}, {b, c}, {d} ]
        // [         {b}, {d, e}, {f} ]
        // [ {a}, {b, c}, {d, e}, {f} ]

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
        assert_eq!(res.get_hop().cloned().collect_vec(), vec!["b".to_string(), "c".to_string()]);
        assert_eq!(res.dimensionality(), 2);
        assert_eq!(res.shape, Bag::from_iter(vec![2, 3]));
        assert_eq!(
            res.contents().cloned().collect_vec(),
            vec!["d".to_string(), "e".to_string(), "f".to_string()]
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
        assert_ne!(lhs_hasher.finish(), rhs_hasher.finish());
    }

    #[test]
    fn diagonal_differences_are_detected_for_up() {
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
        let bfgh = Ortho::new(
            "b".to_owned(),
            "f".to_owned(),
            "g".to_owned(),
            "h".to_owned(),
        );
        let dfgh = Ortho::new(
            "e".to_owned(),
            "d".to_owned(),
            "g".to_owned(),
            "h".to_owned(),
        );

        // 0 1 [a bc d]
        // 1 2

        // 1 2 [b fg h]
        // 2 3 [e dg h]

        assert!(abcd.valid_diagonal_with(&efgh));
        assert!(!abcd.valid_diagonal_with(&bfgh));
        assert!(!abcd.valid_diagonal_with(&dfgh));
    }

    #[test]
    fn zip_up_works_even_if_a_is_d() {
        let l = Ortho::new(
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        );
        let r = Ortho::new(
            "e".to_owned(),
            "f".to_owned(),
            "g".to_owned(),
            "e".to_owned(),
        );
        let corr = vec![
            ("b".to_string(), "f".to_string()),
            ("c".to_string(), "g".to_string()),
        ];

        l.zip_up(&r, &corr);
    }
}
