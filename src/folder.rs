use crate::item::Item;
use crate::line::Line;
use crate::{discontinuity_detector::DiscontinuityDetector, ortho::Ortho, registry::Registry};
use itertools::iproduct;

pub fn single_process(registry: &Registry) -> Registry {
    let new_squares = ffbb(registry);
    registry.add(new_squares)
}

pub fn merge_process(source_answer: &Registry, target_answer: &Registry) -> Registry {
    let detector = DiscontinuityDetector::new(source_answer, target_answer);
    let both = source_answer.union(target_answer);
    let mut check_back = vec![];
    let mut total: u128 = 0;
    let mut hit = 0;
    for line in both.items().iter() {
        let lhss = both.left_of(&line);
        let rhss = both.right_of(&line);

        for (lhs, rhs) in iproduct!(lhss, rhss) {
            total += 1;
            if detector.discontinuity(&lhs, &line, &rhs) {
                hit += 1;
                check_back.push((lhs, line.clone(), rhs));
            }
        }
    }
    if total != 0 {
        dbg!(hit * 100 / total);
    } else {
        dbg!(total);
    }

    let additional_squares = find_additional_squares(&both, check_back);
    both.add(additional_squares)
}

fn find_additional_squares(
    combined_book: &Registry,
    check_back: Vec<(Item, Item, Item)>,
) -> Vec<Ortho> {
    let mut res = vec![];
    for (left, center, right) in check_back.iter() {
        let mut cur = match (left, center, right) {
            (Item::Pair(l), Item::Pair(c), Item::Pair(r)) => handle_lines(combined_book, l, c, r),
            (Item::Pair(_), Item::Pair(_), Item::Square(_)) => todo!(),
            (Item::Pair(_), Item::Square(_), Item::Pair(_)) => todo!(),
            (Item::Pair(_), Item::Square(_), Item::Square(_)) => todo!(),
            (Item::Square(_), Item::Pair(_), Item::Pair(_)) => todo!(),
            (Item::Square(_), Item::Pair(_), Item::Square(_)) => todo!(),
            (Item::Square(_), Item::Square(_), Item::Pair(_)) => todo!(),
            (Item::Square(_), Item::Square(_), Item::Square(_)) => todo!(),
        };
        res.append(&mut cur);
    }

    res
}

fn handle_lines(combined_book: &Registry, left: &Line, center: &Line, right: &Line) -> Vec<Ortho> {
    // left: a-b
    // center: a-c
    // right: c-d
    // a-b
    // |
    // c-d

    // verify b != c
    // verify b -> d
    let mut res = vec![];
    if left.second != center.second {
        if combined_book.contains_line_with(&left.second, &right.second) {
            res.push(Ortho::new(
                left.first.to_string(),
                left.second.to_string(),
                right.first.clone(),
                right.second.clone(),
            ))
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
