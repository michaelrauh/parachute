use std::collections::{HashMap, HashSet};

use itertools::{iproduct, Itertools};

use crate::bag::Bag;
use crate::color::Color;
use crate::line::Line;
use crate::ortho::Ortho;
use crate::registry::Registry;

pub struct DiscontinuityDetector<'a> {
    lines: HashMap<&'a Line, Color>,
    ortho_by_color_origin_and_shape:
        HashMap<Color, HashMap<String, HashMap<Bag<usize>, HashSet<&'a Ortho>>>>,
    line_by_color_and_start: HashMap<Color, HashMap<String, HashSet<&'a Line>>>,
}
impl<'a> DiscontinuityDetector<'a> {
    pub(crate) fn new(source_answer: &'a Registry, target_answer: &'a Registry) -> Self {
        let source_only_lines = source_answer.subtract_lines(&target_answer);
        let target_only_lines = target_answer.subtract_lines(&source_answer);
        let source_only_orthos = source_answer.subtract_orthos(&target_answer);
        let target_only_orthos = target_answer.subtract_orthos(&source_answer);
        let lines = source_only_lines
            .union(&target_only_lines)
            .cloned()
            .map(|i| {
                (
                    i,
                    Self::color_for_line(&source_only_lines, &target_only_lines, &i),
                )
            })
            .collect();
        let obcas: HashMap<Color, HashMap<String, HashMap<Bag<usize>, HashSet<&Ortho>>>> =
            source_only_orthos
                .union(&target_only_orthos)
                .map(|ortho| {
                    let color =
                        Self::color_for_ortho(&source_only_orthos, &target_only_orthos, &ortho);
                    let shape = ortho.shape.clone();
                    let origin = ortho.origin().clone();
                    (color, (origin, shape, ortho))
                })
                .into_group_map()
                .into_iter()
                .fold(HashMap::new(), |mut acc, (color, vec)| {
                    for (origin, shape, ortho) in vec {
                        acc.entry(color.clone())
                            .or_insert_with(HashMap::new)
                            .entry(origin)
                            .or_insert_with(HashMap::new)
                            .entry(shape)
                            .or_insert_with(HashSet::new)
                            .insert(ortho);
                    }
                    acc
                });

        let lbcas: HashMap<Color, HashMap<String, HashSet<&Line>>> = source_only_lines
            .union(&target_only_lines)
            .map(|line| {
                let color = Self::color_for_line(&source_only_lines, &target_only_lines, &line);
                let start = line.first.clone();
                (color, (start, line))
            })
            .into_group_map()
            .into_iter()
            .fold(HashMap::new(), |mut acc, (color, vec)| {
                for (start, line) in vec {
                    acc.entry(color.clone())
                        .or_insert_with(HashMap::new)
                        .entry(start)
                        .or_insert_with(HashSet::new)
                        .insert(line);
                }
                acc
            });

        DiscontinuityDetector {
            lines,
            ortho_by_color_origin_and_shape: obcas,
            line_by_color_and_start: lbcas,
        }
    }

    fn color_for_line(
        source_only: &HashSet<&Line>,
        destination_only: &HashSet<&Line>,
        lhs: &Line,
    ) -> Color {
        if source_only.contains(lhs) {
            Color::Black
        } else if destination_only.contains(lhs) {
            Color::Red
        } else {
            Color::Both
        }
    }

    fn color_for_ortho(
        source_only: &HashSet<&Ortho>,
        destination_only: &HashSet<&Ortho>,
        lhs: &Ortho,
    ) -> Color {
        if source_only.contains(lhs) {
            Color::Black
        } else if destination_only.contains(lhs) {
            Color::Red
        } else {
            Color::Both
        }
    }

    pub(crate) fn l_l_l_discontinuities(&self) -> Vec<(&'a Line, &'a Line, &'a Line)> {
        self.centers()
            .iter()
            .flat_map(|(center_line, center_color)| match center_color {
                Color::Black => self.match_by_discontinuity_black_l_l_l(center_line),
                Color::Red => self.match_by_discontinuity_red_l_l_l(center_line),
                Color::Both => self.match_by_discontinuity_both_l_l_l(center_line),
            })
            .collect()
    }

    pub(crate) fn o_l_o_discontinuities(&self) -> Vec<(&'a Ortho, &'a Line, &'a Ortho)> {
        let centers = self.centers();

        let shapes = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Both)
            .into_iter()
            .flat_map(|origin_map| origin_map.values())
            .flat_map(|shape_map| shape_map.keys())
            .collect::<HashSet<_>>();
        shapes
            .iter()
            .flat_map(move |shape| {
                let shape = *shape;
                let centers = centers.clone();
                centers
                    .iter()
                    .flat_map(move |(center_line, center_color)| match center_color {
                        Color::Black => self.match_by_discontinuity_black_o_l_o(shape, center_line),
                        Color::Red => self.match_by_discontinuity_red_o_l_o(shape, center_line),
                        Color::Both => self.match_by_discontinuity_both_o_l_o(shape, center_line),
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|(l, _c, r)| l.valid_diagonal_with(r))
            .collect()
    }

    fn centers(&self) -> Vec<(&'a Line, &Color)> {
        self.lines
            .iter()
            .map(|(line, color)| (*line, color))
            .collect()
    }

    fn match_by_discontinuity_black_o_l_o(
        &self,
        shape: &Bag<usize>,
        center_line: &'a Line,
    ) -> Vec<(&'a Ortho, &'a Line, &'a Ortho)> {
        // | (Color::Red, Color::Black, Color::Black)
        // | (Color::Red, Color::Black, Color::Both)
        // | (Color::Red, Color::Black, Color::Red)
        // | (Color::Black, Color::Black, Color::Red)
        // | (Color::Both, Color::Black, Color::Red)
        let lhs_black = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let lhs_both = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Both)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let lhs_red = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let rhs_black = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let rhs_both = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Both)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let rhs_red = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        iproduct!(lhs_red, rhs_black.chain(rhs_both).chain(rhs_red.clone()))
            .map(|(l, r)| (*l, center_line, *r))
            .chain(
                iproduct!(lhs_black.chain(lhs_both), rhs_red).map(|(l, r)| (*l, center_line, *r)),
            )
            .collect_vec()
    }

    fn match_by_discontinuity_red_o_l_o(
        &self,
        shape: &Bag<usize>,
        center_line: &'a Line,
    ) -> Vec<(&'a Ortho, &'a Line, &'a Ortho)> {
        // | (Color::Black, Color::Red, Color::Black)
        // | (Color::Black, Color::Red, Color::Red)
        // | (Color::Black, Color::Red, Color::Both)
        // | (Color::Red, Color::Red, Color::Black)
        // | (Color::Both, Color::Red, Color::Black)
        let lhs_black = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let lhs_both = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Both)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let lhs_red = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let rhs_black = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let rhs_both = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Both)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let rhs_red = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        iproduct!(
            lhs_black,
            rhs_red.clone().chain(rhs_black).chain(rhs_both.clone())
        )
        .map(|(l, r)| (*l, center_line, *r))
        .chain(iproduct!(lhs_both.chain(lhs_red), rhs_red).map(|(l, r)| (*l, center_line, *r)))
        .collect_vec()
    }

    fn match_by_discontinuity_both_o_l_o(
        &self,
        shape: &Bag<usize>,
        center_line: &'a Line,
    ) -> Vec<(&'a Ortho, &'a Line, &'a Ortho)> {
        // | (Color::Black, Color::Both, Color::Red)
        // | (Color::Red, Color::Both, Color::Black)

        let lhs_black = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let lhs_red = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let rhs_black = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let rhs_red = self
            .ortho_by_color_origin_and_shape
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|m| m.get(shape))
            .flat_map(|s| s.iter());

        let firsts = iproduct!(lhs_black, rhs_red).map(|(l, r)| (l, center_line, r));

        let seconds = iproduct!(lhs_red, rhs_black).map(|(l, r)| (l, center_line, r));
        firsts
            .chain(seconds)
            .map(|(l, c, r)| (*l, c, *r))
            .collect_vec()
    }

    fn match_by_discontinuity_black_l_l_l(
        &self,
        center_line: &'a Line,
    ) -> Vec<(&'a Line, &'a Line, &'a Line)> {
        // | (Color::Red, Color::Black, Color::Black)
        // | (Color::Red, Color::Black, Color::Both)
        // | (Color::Red, Color::Black, Color::Red)
        // | (Color::Black, Color::Black, Color::Red)
        // | (Color::Both, Color::Black, Color::Red)

        let lhs_black = self
            .line_by_color_and_start
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|s| s.iter());

        let lhs_both = self
            .line_by_color_and_start
            .get(&Color::Both)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|s| s.iter());

        let lhs_red = self
            .line_by_color_and_start
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|s| s.iter());

        let rhs_black = self
            .line_by_color_and_start
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|s| s.iter());

        let rhs_both = self
            .line_by_color_and_start
            .get(&Color::Both)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|s| s.iter());

        let rhs_red = self
            .line_by_color_and_start
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|s| s.iter());

        iproduct!(lhs_red, rhs_black.chain(rhs_both).chain(rhs_red.clone()))
            .map(|(l, r)| (*l, center_line, *r))
            .chain(
                iproduct!(lhs_black.chain(lhs_both), rhs_red).map(|(l, r)| (*l, center_line, *r)),
            )
            .collect_vec()
    }

    fn match_by_discontinuity_red_l_l_l(
        &self,
        center_line: &'a Line,
    ) -> Vec<(&'a Line, &'a Line, &'a Line)> {
        // | (Color::Black, Color::Red, Color::Black)
        // | (Color::Black, Color::Red, Color::Red)
        // | (Color::Black, Color::Red, Color::Both)
        // | (Color::Red, Color::Red, Color::Black)
        // | (Color::Both, Color::Red, Color::Black)

        let lhs_black = self
            .line_by_color_and_start
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|s| s.iter());

        let lhs_both = self
            .line_by_color_and_start
            .get(&Color::Both)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|s| s.iter());

        let lhs_red = self
            .line_by_color_and_start
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|s| s.iter());

        let rhs_black = self
            .line_by_color_and_start
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|s| s.iter());

        let rhs_both = self
            .line_by_color_and_start
            .get(&Color::Both)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|s| s.iter());

        let rhs_red = self
            .line_by_color_and_start
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|s| s.iter());

        iproduct!(
            lhs_black,
            rhs_red
                .clone()
                .chain(rhs_black.clone())
                .chain(rhs_both.clone())
        )
        .map(|(l, r)| (*l, center_line, *r))
        .chain(iproduct!(lhs_red.chain(lhs_both), rhs_black).map(|(l, r)| (*l, center_line, *r)))
        .collect_vec()
    }

    fn match_by_discontinuity_both_l_l_l(
        &self,
        center_line: &'a Line,
    ) -> Vec<(&'a Line, &'a Line, &'a Line)> {
        // | (Color::Black, Color::Both, Color::Red)
        // | (Color::Red, Color::Both, Color::Black)

        let lhs_black = self
            .line_by_color_and_start
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|s| s.iter());

        let lhs_red = self
            .line_by_color_and_start
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.first))
            .flat_map(|s| s.iter());

        let rhs_black = self
            .line_by_color_and_start
            .get(&Color::Black)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|s| s.iter());

        let rhs_red = self
            .line_by_color_and_start
            .get(&Color::Red)
            .into_iter()
            .flat_map(|m| m.get(&center_line.second))
            .flat_map(|s| s.iter());

        iproduct!(lhs_black, rhs_red)
            .map(|(l, r)| (*l, center_line, *r))
            .chain(iproduct!(lhs_red, rhs_black).map(|(l, r)| (*l, center_line, *r)))
            .collect_vec()
    }
}
