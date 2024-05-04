use crate::color::Color;
use crate::item::Item;
use crate::registry::Registry;

pub struct DiscontinuityDetector {
    source_only: Registry,
    destination_only: Registry,
}
impl DiscontinuityDetector {
    pub(crate) fn new(source_answer: &Registry, target_answer: &Registry) -> Self {
        DiscontinuityDetector {
            source_only: source_answer.minus(target_answer),
            destination_only: target_answer.minus(source_answer),
        }
    }

    pub(crate) fn discontinuity(&self, lhs: &Item, line: &Item, rhs: &Item) -> bool {
        match (self.color(lhs), self.color(line), self.color(rhs)) {
            (Color::Black, Color::Black, Color::Red) => true,
            (Color::Black, Color::Red, Color::Black) => true,
            (Color::Black, Color::Red, Color::Red) => true,
            (Color::Black, Color::Red, Color::Both) => true,
            (Color::Black, Color::Both, Color::Red) => true,
            (Color::Red, Color::Black, Color::Black) => true,
            (Color::Red, Color::Black, Color::Both) => true,
            (Color::Red, Color::Black, Color::Red) => true,
            (Color::Red, Color::Red, Color::Black) => true,
            (Color::Red, Color::Both, Color::Black) => true,
            (Color::Both, Color::Black, Color::Red) => true,
            (Color::Both, Color::Red, Color::Black) => true,
            _ => false,
        }
    }

    fn color(&self, lhs: &Item) -> Color {
        if self.source_only.contains(lhs) {
            Color::Black
        } else if self.destination_only.contains(lhs) {
            Color::Red
        } else {
            Color::Both
        }
    }
}
