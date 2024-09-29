use std::collections::HashSet;
use std::vec;

use crate::line::Line;
use crate::{discontinuity_detector::DiscontinuityDetector, ortho::Ortho, registry::Registry};
use itertools::{iproduct, Itertools};

pub fn single_process(registry: &Registry) -> Registry {
    let new_squares = ffbb(registry);
    let r = registry.add(new_squares.clone());
    fold_up_by_origin_repeatedly(r, new_squares)
}

pub fn merge_process(source_answer: &Registry, target_answer: &Registry) -> Registry {
    let detector = DiscontinuityDetector::new(source_answer.to_owned(), target_answer.to_owned());
    dbg!("unioning");
    let both = source_answer.union(target_answer);

    dbg!("detecting line discontinuities");
    let lll_discontinuities = detector.l_l_l_discontinuities();

    dbg!("detecting ortho discontinuities");
    let olo_discontinuities = detector.o_l_o_discontinuities();

    let additional_squares = find_additional_squares_from_l_l_l(&both, lll_discontinuities);
    let more_squares = find_additional_squares_from_o_l_o(&both, olo_discontinuities);
    let all_squares: Vec<Ortho> = additional_squares.into_iter().chain(more_squares).collect();
    let r = both.add(all_squares.clone());
    fold_up_by_origin_repeatedly(r, all_squares)
}

fn fold_up_by_origin_repeatedly(r: Registry, new_squares: Vec<Ortho>) -> Registry {
    dbg!(new_squares.len());
    std::iter::successors(
        Some((r, new_squares)),
        |(current_registry, current_squares)| {
            if current_squares.is_empty() {
                None
            } else {
                let folded_squares = fold_up_by_origin(current_registry, current_squares.clone());
                Some((current_registry.add(folded_squares.clone()), folded_squares))
            }
        },
    )
    .last()
    .unwrap()
    .0
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
                        .map(|other| handle_connection(r, &&ortho, &&other))
                        .flatten()
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn find_additional_squares_from_l_l_l(
    combined_book: &Registry,
    check_back: Vec<(Line, Line, Line)>,
) -> Vec<Ortho> {
    check_back
        .iter()
        .flat_map(|(l, c, r)| handle_lines(combined_book, l, c, r))
        .collect_vec()
}

fn find_additional_squares_from_o_l_o<I>(combined_book: &Registry, check_back: I) -> Vec<Ortho>
where
    I: Iterator<Item = (Ortho, Line, Ortho)>,
{
    check_back
        .flat_map(|(l, _c, r)| handle_connection(combined_book, &l, &r))
        .collect_vec()
}

fn handle_connection(registry: &Registry, l: &Ortho, r: &Ortho) -> Vec<Ortho> {
    // todo make sure only base orthos are passed in, or check here
    // left: ortho with origin (for now) connected to the other (origin = a)
    // center: a-b
    // right ortho.origin = b
    // assumption: passed in orthos have the same shape
    // assumption: passed in orthos have valid diagonals

    let potential_corresponding_axes = find_potential_correspondences(registry, l, r);
    let ans = potential_corresponding_axes
        .into_iter()
        .flat_map(|correspondence| {
            attempt_combine_up_by_corresponding_configuration(registry, l, r, correspondence)
        })
        .collect_vec();

    ans
}

fn attempt_combine_up_by_corresponding_configuration(
    registry: &Registry,
    l: &Ortho,
    r: &Ortho,
    correspondence: Vec<(String, String)>,
) -> Vec<Ortho> {
    if all_other_connections_work(registry, l, r, &correspondence) {
        vec![l.zip_up(r, &correspondence)]
    } else {
        vec![]
    }
}

fn all_other_connections_work(
    registry: &Registry,
    l: &Ortho,
    r: &Ortho,
    correspondence: &[(String, String)],
) -> bool {
    l.contents()
        .iter()
        .all(|left_word| l.connection_works(left_word.to_string(), registry, correspondence, &r))
}

fn find_potential_correspondences(
    registry: &Registry,
    l: &Ortho,
    r: &Ortho,
) -> Vec<Vec<(String, String)>> {
    let left_axes = l.get_hop();
    let right_axes = r.get_hop();
    let potentials: Vec<(String, String)> = iproduct!(left_axes, right_axes)
        .filter(|(left_try, right_try)| registry.contains_line_with(left_try, right_try))
        .collect();

    if sufficient_axes_to_cover(&potentials, l) {
        combobulate_axes(potentials)
    } else {
        vec![]
    }
}

fn combobulate_axes(potentials: Vec<(String, String)>) -> Vec<Vec<(String, String)>> {
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
            let unique_lefts = combo.iter().map(|(left, _)| left).collect::<HashSet<_>>();
            let unique_rights = combo.iter().map(|(_, right)| right).collect::<HashSet<_>>();
            unique_lefts.len() == num_axes && unique_rights.len() == num_axes
        })
        .collect()
}

fn sufficient_axes_to_cover(potentials: &[(String, String)], l: &Ortho) -> bool {
    let required = l.dimensionality();

    let (left, right): (HashSet<_>, HashSet<_>) = potentials
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
        let r = Registry::from_text("a b c d. a c. b d.", "first.txt", 1);
        let res = single_process(&r);

        assert_eq!(
            res.squares,
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

        let r = Registry::from_text(
            "a b. c d. a c. b d. e f. g h. e g. f h. a e. b f. c g. d h.",
            "first.txt",
            1,
        );
        let res = single_process(&r);

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
        assert!(res.squares.contains(&expected_ortho))
    }

    #[test]
    fn test_merge_process_discovers_squares_from_lines() {
        let left_registry = single_process(&Registry::from_text("a b. c d.", "first.txt", 1));
        let right_registry = single_process(&Registry::from_text("a c. b d.", "second.txt", 2));
        let res = merge_process(&left_registry, &right_registry);

        assert_eq!(
            res.squares,
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
        let left_registry = single_process(&Registry::from_text(
            "a b. c d. a c. b d. a e. b f. c g. d h.",
            "first.txt",
            1,
        ));
        let right_registry =
            single_process(&Registry::from_text("e f. g h. e g. f h.", "second.txt", 2));

        let res = merge_process(&left_registry, &right_registry);
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

        assert!(res.squares.contains(&expected_ortho))
    }

    #[test]
    fn test_merge_process_sifts_down_by_origin_for_up_dimension() {
        // a b  e f
        // c d  g h

        let left_registry = single_process(&Registry::from_text(
            "a b. a c. b d. a e. b f. c g. d h.",
            "first.txt",
            1,
        ));
        let right_registry = single_process(&Registry::from_text(
            "c d. e f. g h. e g. f h.",
            "second.txt",
            2,
        ));

        let res = merge_process(&left_registry, &right_registry);
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

        assert!(res.squares.contains(&expected_ortho))
    }
}
