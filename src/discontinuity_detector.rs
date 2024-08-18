use std::collections::HashMap;

use crate::color::Color;
use crate::item::Item;
use crate::registry::Registry;

pub struct DiscontinuityDetector {
    item_to_color: HashMap<Item, Color>,
}
impl DiscontinuityDetector {
    pub(crate) fn new(source_answer: Registry, target_answer: Registry) -> Self {
        let source_only = source_answer.minus(&target_answer);
        let destination_only = target_answer.minus(&source_answer);
        let item_to_color: HashMap<Item, Color> = source_answer
            .clone()
            .union(&target_answer)
            .items()
            .iter()
            .cloned()
            .map(|i| (i.clone(), Self::color(&source_only, &destination_only, &i)))
            .collect();
        DiscontinuityDetector { item_to_color }
    }

    pub(crate) fn discontinuity(&self, lhs: &Item, line: &Item, rhs: &Item) -> bool {
        matches!(
            (
                self.color_item(lhs),
                self.color_item(line),
                self.color_item(rhs)
            ),
            (Color::Black, Color::Black, Color::Red)
                | (Color::Black, Color::Red, Color::Black)
                | (Color::Black, Color::Red, Color::Red)
                | (Color::Black, Color::Red, Color::Both)
                | (Color::Black, Color::Both, Color::Red)
                | (Color::Red, Color::Black, Color::Black)
                | (Color::Red, Color::Black, Color::Both)
                | (Color::Red, Color::Black, Color::Red)
                | (Color::Red, Color::Red, Color::Black)
                | (Color::Red, Color::Both, Color::Black)
                | (Color::Both, Color::Black, Color::Red)
                | (Color::Both, Color::Red, Color::Black)
        )
    }

    fn color_item(&self, item: &Item) -> Color {
        self.item_to_color[item].clone()
    }

    fn color(source_only: &Registry, destination_only: &Registry, lhs: &Item) -> Color {
        if source_only.contains(lhs) {
            Color::Black
        } else if destination_only.contains(lhs) {
            Color::Red
        } else {
            Color::Both
        }
    }
}
