use std::collections::{HashSet, VecDeque};
use std::vec;

use crate::line::Line;
use crate::{discontinuity_detector::DiscontinuityDetector, ortho::Ortho, registry::Registry};
use itertools::{iproduct, Itertools};

pub fn single_process(registry: &mut Registry) {
    let mut new_squares = ffbb(registry);
    let total = new_squares.len();

    for (i, square) in new_squares.drain(..).enumerate() {
        let percent_done = (i as f64 / total as f64) * 100.0;
        dbg!(percent_done);
        if let Some(added) = registry.add_one(square) { // revisit registry and ortho for duplication
            fold_up_by_origin_repeatedly(registry, added)
        }
    }
}

pub fn merge_process(source_answer: &mut Registry, target_answer: &mut Registry) {
    let (additional_squares, more_squares) = {
        let detector = DiscontinuityDetector::new(source_answer, target_answer);

        dbg!("detecting line discontinuities");
        let lll_discontinuities = detector.l_l_l_discontinuities();

        dbg!("detecting ortho discontinuities");
        let olo_discontinuities = detector.o_l_o_discontinuities();

        let additional_squares =
            find_additional_squares_from_l_l_l(source_answer, lll_discontinuities);
        let more_squares = find_additional_squares_from_o_l_o(source_answer, olo_discontinuities);

        (additional_squares, more_squares)
    };

    dbg!("unioning");
    for line in target_answer.pairs.drain() { // consider drain, revisit clone, look for more vecs
        source_answer.add_line(line);
    }

    for square in target_answer.squares.drain() {
        source_answer.add_one(square);
    }

    let total = additional_squares.len() + more_squares.len();
    for (i, square) in additional_squares.into_iter().chain(more_squares).enumerate() {
        let percent_done = (i as f64 / total as f64) * 100.0;
        dbg!(percent_done);
        if let Some(added) = source_answer.add_one(square.clone()) {
            fold_up_by_origin_repeatedly(source_answer, added);
        }
    }
}

fn fold_up_by_origin_repeatedly(r: &mut Registry, new_square: Ortho) {
    let mut queue: VecDeque<Ortho> = VecDeque::new();
    queue.push_back(new_square);
    while let Some(ortho) = queue.pop_front() {
        dbg!(&queue.len());
        let mut folded_orthos: Vec<Ortho> = fold_up_by_origin(r, &ortho).collect();

        for folded_ortho in folded_orthos.drain(..) {
            if let Some(added) = r.add_one(folded_ortho) { 
                queue.push_back(added);
            }
        }
    }
}

fn fold_up_by_origin<'a>(r: &'a Registry, ortho: &'a Ortho) -> impl Iterator<Item = Ortho> + 'a {
    r.forward(ortho.origin()).iter().flat_map(move |second| {
        r.squares_with_origin(second)
            .into_iter()
            .filter(move |o| o.shape == ortho.shape && o.valid_diagonal_with(ortho))
            .filter_map(move |other| handle_connection(r, ortho, other))
    })
}

fn find_additional_squares_from_l_l_l(
    combined_book: &Registry,
    check_back: Vec<(&Line, &Line, &Line)>,
) -> Vec<Ortho> {
    check_back
        .into_iter()
        .filter_map(move |(l, c, r)| handle_lines(combined_book, l, c, r))
        .collect()
}

fn find_additional_squares_from_o_l_o(
    combined_book: &Registry,
    check_back: Vec<(&Ortho, &Line, &Ortho)>,
) -> Vec<Ortho> {
    check_back
        .into_iter()
        .filter_map(move |(l, _c, r)| handle_connection(combined_book, l, r))
        .collect()
}

fn handle_connection(registry: &Registry, l: &Ortho, r: &Ortho) -> Option<Ortho> {
    if let Some(potential_corresponding_axes) = find_potential_correspondences(registry, l, r) {
        for correspondence in potential_corresponding_axes {
            if let Some(new_ortho) =
                attempt_combine_up_by_corresponding_configuration(registry, l, r, correspondence)
            {
                return Some(new_ortho);
            }
        }
    }
    None
}

fn attempt_combine_up_by_corresponding_configuration(
    registry: &Registry,
    l: &Ortho,
    r: &Ortho,
    correspondence: Vec<(String, String)>,
) -> Option<Ortho> {
    if all_other_connections_work(registry, l, r, &correspondence) {
        Some(l.zip_up(r, &correspondence))
    } else {
        None
    }
}

fn all_other_connections_work(
    registry: &Registry,
    l: &Ortho,
    r: &Ortho,
    correspondence: &[(String, String)],
) -> bool {
    l.contents()
        .all(|left_word| l.connection_works(left_word.to_string(), registry, correspondence, &r))
}

fn find_potential_correspondences(
    registry: &Registry,
    l: &Ortho,
    r: &Ortho,
) -> Option<Vec<Vec<(String, String)>>> {
    let left_axes = l.get_hop().collect_vec();
    let right_axes = r.get_hop().collect_vec();
    let potentials: Vec<(&String, &String)> = iproduct!(left_axes, right_axes)
        .filter(|(left_try, right_try)| registry.contains_line_with(left_try, right_try))
        .collect();

    if sufficient_axes_to_cover(&potentials, l) {
        Some(combobulate_axes(potentials))
    } else {
        None
    }
}

fn combobulate_axes(potentials: Vec<(&String, &String)>) -> Vec<Vec<(String, String)>> {
    let num_axes = potentials
        .iter()
        .map(|(left, _)| left)
        .collect::<HashSet<_>>()
        .len();

    potentials
        .iter()
        .cloned()
        .combinations(num_axes)
        .filter(|combo| {
            let mut unique_lefts = HashSet::with_capacity(num_axes);
            let mut unique_rights = HashSet::with_capacity(num_axes);

            for (left, right) in combo {
                unique_lefts.insert(left);
                unique_rights.insert(right);
            }

            unique_lefts.len() == num_axes && unique_rights.len() == num_axes
        })
        .map(|combo| {
            combo
                .into_iter()
                .map(|(left, right)| (left.clone(), right.clone()))
                .collect()
        })
        .collect()
}

fn sufficient_axes_to_cover(potentials: &[(&String, &String)], l: &Ortho) -> bool {
    let required = l.dimensionality();

    let (left, right): (HashSet<&String>, HashSet<&String>) = potentials
        .iter()
        .map(|(left_potential, right_potential)| (left_potential, right_potential))
        .unzip();

    left.len() == required && right.len() == required
}

fn handle_lines(registry: &Registry, left: &Line, center: &Line, right: &Line) -> Option<Ortho> {
    // left: a-b
    // center: a-c
    // right: c-d
    // a-b
    // |
    // c-d

    // verify b != c
    // verify b -> d

    if left.second != center.second && registry.contains_line_with(&left.second, &right.second) {
        Some(Ortho::new(
            left.first.to_string(),
            left.second.to_string(),
            right.first.clone(),
            right.second.clone(),
        ))
    } else {
        None
    }
}

fn ffbb(book: &Registry) -> Vec<Ortho> {
    let mut res = vec![];
    for line in book.pairs.clone() {
        let a = line.first;
        let b = line.second;
        for d in book.forward(&b) {
            for c in book.backward(d) {
                if &b != c && book.backward(c).contains(&a) {
                    res.push(Ortho::new(
                        a.to_string(),
                        b.to_string(),
                        c.clone(),
                        d.clone(),
                    ))
                }
            }
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{folder::merge_process, ortho::Ortho, registry::Registry};

    use super::single_process;

    #[test]
    fn test_single_process_discovers_squares() {
        let mut r = Registry::from_text("a b c d. a c. b d.", "first.txt", 1);
        single_process(&mut r);

        assert_eq!(
            r.squares,
            vec![Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string()
            )]
            .into_iter()
            .collect::<HashSet<_>>()
        )
    }

    #[test]
    fn test_single_process_sifts_down_by_origin_for_up_dimension() {
        // a b  e f
        // c d  g h

        let mut r = Registry::from_text(
            "a b. c d. a c. b d. e f. g h. e g. f h. a e. b f. c g. d h.",
            "first.txt",
            1,
        );
        single_process(&mut r);

        let abcd = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        let efgh = Ortho::new(
            "e".to_string(),
            "f".to_string(),
            "g".to_string(),
            "h".to_string(),
        );
        let expected_ortho = abcd.zip_up(
            &efgh,
            &[
                ("b".to_string(), "f".to_string()),
                ("c".to_string(), "g".to_string()),
            ],
        );
        assert!(r.squares.contains(&expected_ortho))
    }

    #[test]
    fn test_merge_process_discovers_squares_from_lines() {
        let mut left_registry = Registry::from_text("a b. c d.", "first.txt", 1);
        let mut right_registry = Registry::from_text("a c. b d.", "second.txt", 2);

        single_process(&mut left_registry);
        single_process(&mut right_registry);
        merge_process(&mut left_registry, &mut right_registry);

        assert_eq!(
            left_registry.squares,
            vec![Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string()
            )]
            .into_iter()
            .collect::<HashSet<_>>()
        )
    }

    #[test]
    fn test_merge_process_discovers_squares_from_squares() {
        let mut left_registry =
            Registry::from_text("a b. c d. a c. b d. a e. b f. c g. d h.", "first.txt", 1);

        let mut right_registry = Registry::from_text("e f. g h. e g. f h.", "second.txt", 2);
        single_process(&mut left_registry);

        single_process(&mut right_registry);

        merge_process(&mut left_registry, &mut right_registry);
        let expected_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        )
        .zip_up(
            &Ortho::new(
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
                "h".to_string(),
            ),
            &[
                ("b".to_string(), "f".to_string()),
                ("c".to_string(), "g".to_string()),
            ],
        );

        assert!(left_registry.squares.contains(&expected_ortho))
    }

    #[test]
    fn test_merge_process_sifts_down_by_origin_for_up_dimension() {
        // a b  e f
        // c d  g h

        let mut left_registry =
            Registry::from_text("a b. a c. b d. a e. b f. c g. d h.", "first.txt", 1);

        single_process(&mut left_registry);
        let mut right_registry = Registry::from_text("c d. e f. g h. e g. f h.", "second.txt", 2);

        single_process(&mut right_registry);

        merge_process(&mut left_registry, &mut right_registry);
        let expected_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        )
        .zip_up(
            &Ortho::new(
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
                "h".to_string(),
            ),
            &[
                ("b".to_string(), "f".to_string()),
                ("c".to_string(), "g".to_string()),
            ],
        );

        assert!(left_registry.squares.contains(&expected_ortho))
    }
}