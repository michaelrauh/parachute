use std::collections::HashSet;
use std::vec;

use crate::line::Line;
use crate::{discontinuity_detector::DiscontinuityDetector, ortho::Ortho, registry::Registry};
use itertools::{iproduct, Itertools};

pub fn single_process(registry: &mut Registry) {
    let new_squares = ffbb(registry);
    let added_squares = registry.add(new_squares);
    fold_up_by_origin_repeatedly(registry, added_squares)
}

pub fn merge_process(source_answer: &mut Registry, target_answer: &Registry) {
    let detector = DiscontinuityDetector::new(source_answer, target_answer);
    dbg!("unioning");
    source_answer.add_lines(target_answer.pairs.iter().cloned().collect::<Vec<_>>());

    source_answer.add(
        target_answer
            .squares
            .clone()
            .into_iter()
            .collect::<Vec<_>>(),
    );

    dbg!("detecting line discontinuities");
    let lll_discontinuities = detector.l_l_l_discontinuities();

    dbg!("detecting ortho discontinuities");
    let olo_discontinuities = detector.o_l_o_discontinuities();

    let additional_squares = find_additional_squares_from_l_l_l(source_answer, lll_discontinuities);
    let more_squares = find_additional_squares_from_o_l_o(source_answer, olo_discontinuities);
    let all_squares: Vec<Ortho> = additional_squares.into_iter().chain(more_squares).collect();
    let added_squares = source_answer.add(all_squares);
    fold_up_by_origin_repeatedly(source_answer, added_squares)
}

fn fold_up_by_origin_repeatedly(r: &mut Registry, new_squares: Vec<Ortho>) {
    dbg!(new_squares.len());
    let mut current_squares = new_squares;

    while !current_squares.is_empty() {
        let folded_squares = fold_up_by_origin(r, current_squares);
        current_squares = r.add(folded_squares);
    }
}

fn fold_up_by_origin(r: &Registry, new_squares: Vec<Ortho>) -> Vec<Ortho> {
    dbg!(new_squares.len());
    new_squares
        .iter()
        .flat_map(|ortho| {
            r.forward(ortho.origin())
                .iter()
                .flat_map(|second| {
                    r.squares_with_origin(second)
                        .into_iter()
                        .filter(|o| o.shape == ortho.shape)
                        .filter(|o| o.valid_diagonal_with(ortho))
                        .map(|other| handle_connection(r, &&ortho, &&other).into_iter().flatten())
                        .flatten()
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn find_additional_squares_from_l_l_l(
    combined_book: &Registry,
    check_back: Vec<(&Line, &Line, &Line)>,
) -> Vec<Ortho> {
    check_back
        .iter()
        .flat_map(|(l, c, r)| handle_lines(combined_book, l, c, r))
        .collect_vec()
}

fn find_additional_squares_from_o_l_o(
    combined_book: &Registry,
    check_back: Vec<(&Ortho, &Line, &Ortho)>,
) -> Vec<Ortho> {
    check_back
        .iter()
        .flat_map(|(l, _c, r)| handle_connection(combined_book, &l, &r).into_iter().flatten())
        .collect_vec()
}

fn handle_connection(registry: &Registry, l: &Ortho, r: &Ortho) -> Option<Vec<Ortho>> {
    // todo make sure only base orthos are passed in, or check here
    // left: ortho with origin (for now) connected to the other (origin = a)
    // center: a-b
    // right ortho.origin = b
    // assumption: passed in orthos have the same shape
    // assumption: passed in orthos have valid diagonals
    
    let potential_corresponding_axes = find_potential_correspondences(registry, l, r);
    if let Some(potential_corresponding_axes) = potential_corresponding_axes {
        let ans = potential_corresponding_axes
            .into_iter()
            .flat_map(|correspondence| {
                attempt_combine_up_by_corresponding_configuration(registry, l, r, correspondence)
            })
            .collect_vec();

        Some(ans)
    } else {
        None
    }
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
        .map(|combo| combo.into_iter().map(|(left, right)| (left.clone(), right.clone())).collect())
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

fn handle_lines(registry: &Registry, left: &Line, center: &Line, right: &Line) -> Vec<Ortho> {
    // left: a-b
    // center: a-c
    // right: c-d
    // a-b
    // |
    // c-d

    // verify b != c
    // verify b -> d

    if left.second != center.second && registry.contains_line_with(&left.second, &right.second) {
        vec![Ortho::new(
            left.first.to_string(),
            left.second.to_string(),
            right.first.clone(),
            right.second.clone(),
        )]
    } else {
        vec![]
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
        merge_process(&mut left_registry, &right_registry);

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

        merge_process(&mut left_registry, &right_registry);
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

        merge_process(&mut left_registry, &right_registry);
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
