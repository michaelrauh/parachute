use std::collections::HashSet;
use std::vec;

use crate::hit_counter::HitCounter;
use crate::item::Item;
use crate::line::Line;
use crate::{discontinuity_detector::DiscontinuityDetector, ortho::Ortho, registry::Registry};
use itertools::{iproduct, Itertools};

pub fn single_process(registry: &Registry) -> Registry {
    let new_squares = ffbb(registry);
    let r = registry.add(new_squares.clone());
    fold_up_by_origin_repeatedly(r, new_squares)
}

fn fold_up_by_origin_repeatedly(r: Registry, new_squares: Vec<Ortho>) -> Registry {
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
    new_squares
        .into_iter()
        .flat_map(|ortho| {
            r.squares_with_origin(ortho.origin())
                .into_iter()
                .filter_map(move |other| {
                    if let crate::item::Item::Square(s) = other {
                        Some(handle_connection(r, &&ortho, &s))
                    } else {
                        None
                    }
                })
                .flatten()
        })
        .collect()
}

pub fn merge_process(source_answer: &Registry, target_answer: &Registry) -> Registry {
    let detector = DiscontinuityDetector::new(source_answer, target_answer);
    let both = source_answer.union(target_answer);
    let mut check_back = vec![];
    let mut hit_counter = HitCounter::default();
    for line in both.items().iter() {
        let lhss = both.left_of(line);
        let rhss = both.right_of(line);

        for (lhs, rhs) in iproduct!(lhss, rhss) {
            hit_counter.swing();
            if detector.discontinuity(&lhs, line, &rhs) {
                hit_counter.hit();
                check_back.push((lhs, line.clone(), rhs));
            }
        }
    }
    dbg!(hit_counter.ratio());

    let additional_squares = find_additional_squares(&both, check_back);
    both.add(additional_squares)
}

fn find_additional_squares(
    combined_book: &Registry,
    check_back: Vec<(Item, Item, Item)>,
) -> Vec<Ortho> {
    check_back
        .iter()
        .flat_map(|(left, center, right)| match (left, center, right) {
            (Item::Pair(l), Item::Pair(c), Item::Pair(r)) => handle_lines(combined_book, l, c, r),
            (Item::Pair(_), Item::Pair(_), Item::Square(_)) => vec![],
            (Item::Pair(_), Item::Square(_), Item::Pair(_)) => unreachable!(),
            (Item::Pair(_), Item::Square(_), Item::Square(_)) => unreachable!(),
            (Item::Square(_), Item::Pair(_), Item::Pair(_)) => vec![],
            (Item::Square(l), Item::Pair(_), Item::Square(r)) => {
                handle_connection(combined_book, l, r)
            }
            (Item::Square(_), Item::Square(_), Item::Pair(_)) => unreachable!(),
            (Item::Square(_), Item::Square(_), Item::Square(_)) => unreachable!(),
        })
        .collect()
}

// todo make sure it isnt necessary to separate out why this was called. There should be separate methods or types for by origin, hop, contents
// tell don't ask - look at the ifs and push them down

fn handle_connection(registry: &Registry, l: &&Ortho, r: &&Ortho) -> Vec<Ortho> {
    // left: ortho with origin (for now) connected to the other (origin = a)
    // center: a-b
    // right ortho.origin = b

    let potential_corresponding_axes = find_potential_correspondences(registry, l, r);
    potential_corresponding_axes
        .into_iter()
        .flat_map(|correspondence| {
            attempt_combine_up_by_corresponding_configuration(registry, l, r, correspondence)
        })
        .collect_vec()
}

fn attempt_combine_up_by_corresponding_configuration(
    registry: &Registry,
    l: &&Ortho,
    r: &&Ortho,
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
    l.contents().iter().all(|other_connection| {
        l.connection_works(other_connection.to_string(), registry, correspondence, &r)
    })
}

fn find_potential_correspondences(
    registry: &Registry,
    l: &&Ortho,
    r: &&Ortho,
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
        for d in book.forward(b.clone()) {
            for c in book.backward(d.clone()) {
                if b != c && book.backward(c.clone()).contains(&a) {
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
