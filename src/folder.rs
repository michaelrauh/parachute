use crate::item::Item::Line;
use crate::{discontinuity_detector::DiscontinuityDetector, ortho::Ortho, registry::Registry};
use itertools::iproduct;

pub fn single_process(registry: &Registry) -> Registry {
    let new_squares = ffbb(registry);
    registry.add(new_squares);
    registry.clone()
}

pub fn merge_process(source_answer: &Registry, target_answer: &Registry) -> Registry {
    let detector = DiscontinuityDetector::new(source_answer, target_answer);
    let both = source_answer.union(target_answer);
    let mut check_back = vec![];
    for line in both.iter() {
        let lhss = both.left_of(&line);
        let rhss = both.right_of(&line);

        for (lhs, rhs) in iproduct!(lhss, rhss) {
            if detector.discontinuity(&lhs, &line, &rhs) {
                check_back.push((lhs, line.clone(), rhs));
            }
        }
    }

    let additional_squares = find_additional_squares(both, check_back);
    source_answer
        .union(target_answer)
        .add(additional_squares)
}

fn find_additional_squares(
    combined_book: Registry,
    check_back: Vec<(crate::item::Item, crate::item::Item, crate::item::Item)>,
) -> Vec<Ortho> {
    // left: a-b
    // center: a-c
    // right: c-d
    // a-b
    // |
    // c-d

    // verify b != c
    // verify b -> d
    let mut res = vec![];
    for (l, c, r) in check_back.iter() {
        if let (Line(left), Line(center), Line(right)) = (l, c, r) {
            if left.second != center.second {
                if combined_book
                    .forward(left.second.clone())
                    .contains(&right.second)
                {
                    res.push(Ortho::new(
                        left.first.to_string(),
                        left.second.to_string(),
                        right.first.clone(),
                        right.second.clone(),
                    ))
                }
            }
        } else {
            todo!()
        }
    }

    res
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
