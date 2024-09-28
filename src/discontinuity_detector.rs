use std::collections::HashMap;

use itertools::{iproduct, Itertools};

use crate::bag::Bag;
use crate::color::Color;
use crate::line::Line;
use crate::ortho::Ortho;
use crate::registry::Registry;

pub struct DiscontinuityDetector {
    lines: HashMap<Line, Color>,
    orthos: HashMap<Ortho, Color>,
    source: Registry,
    target: Registry,
}
impl DiscontinuityDetector {
    pub(crate) fn new(source_answer: Registry, target_answer: Registry) -> Self {
        let source_only = source_answer.minus(&target_answer);
        let destination_only = target_answer.minus(&source_answer);
        let union = &source_answer.clone().union(&target_answer);
        let lines = union
            .lines()
            .iter()
            .cloned()
            .map(|i| {
                (
                    i.clone(),
                    Self::color_for_line(&source_only, &destination_only, &i),
                )
            })
            .collect();
        let orthos = union
            .orthos()
            .iter()
            .cloned()
            .map(|i| {
                (
                    i.clone(),
                    Self::color_for_ortho(&source_only, &destination_only, &i),
                )
            })
            .collect();
        DiscontinuityDetector {
            lines,
            orthos,
            source: source_answer,
            target: target_answer,
        }
    }

    fn color_for_line(source_only: &Registry, destination_only: &Registry, lhs: &Line) -> Color {
        if source_only.contains_line(lhs) {
            Color::Black
        } else if destination_only.contains_line(lhs) {
            Color::Red
        } else {
            Color::Both
        }
    }

    fn color_for_ortho(source_only: &Registry, destination_only: &Registry, lhs: &Ortho) -> Color {
        if source_only.contains_ortho(lhs) {
            Color::Black
        } else if destination_only.contains_ortho(lhs) {
            Color::Red
        } else {
            Color::Both
        }
    }

    pub(crate) fn l_l_l_discontinuities(&self) -> Vec<(Line, Line, Line)> {
        self.centers()
            .iter()
            .flat_map(|(center_line, center_color)| {
                let lhss = self.line_left_of(center_line);
                let rhss = self.line_right_of(center_line);

                match center_color {
                    Color::Black => {
                        self.match_by_discontinuity_black_l_l_l(center_line, lhss, rhss)
                    }
                    Color::Red => self.match_by_discontinuity_red_l_l_l(center_line, lhss, rhss),
                    Color::Both => self.match_by_discontinuity_both_l_l_l(center_line, lhss, rhss),
                }
            })
            .collect()
    }

    pub(crate) fn o_l_o_discontinuities(&self) -> impl Iterator<Item = (Ortho, Line, Ortho)> + '_ {
        let centers = self.centers().to_owned();
        let of = centers.len();
        let left_shapes = self
        .source
        .count_by_shape()
        .iter()
        .map(|(s, _c)| s.clone())
        .collect_vec();

        let right_shapes = self
        .target
        .count_by_shape()
        .iter()
        .map(|(s, _c)| s.clone())
        .collect_vec();

        let shapes = left_shapes.into_iter().filter(move |s| right_shapes.contains(s));

        shapes.flat_map(move |shape| {
            centers
                .clone()
                .into_iter()
                .enumerate()
                .flat_map(move |(i, (center_line, center_color))| {
                    dbg!(i, of);
                    let lhss = self.ortho_left_of(&center_line, &shape);
                    let rhss = self.ortho_right_of(&center_line, &shape);

                    match center_color {
                        Color::Black => {
                            self.match_by_discontinuity_black_o_l_o(&center_line, lhss, rhss)
                        }
                        Color::Red => {
                            self.match_by_discontinuity_red_o_l_o(&center_line, lhss, rhss)
                        }
                        Color::Both => {
                            self.match_by_discontinuity_both_o_l_o(&center_line, lhss, rhss)
                        }
                    }
                })
                .filter(|(l, _c, r)| l.valid_diagonal_with(r))
        })
    }

    fn centers(&self) -> Vec<(Line, Color)> {
        self.lines
            .iter()
            .map(|(l, r)| (l.clone(), r.clone()))
            .collect()
    }

    fn ortho_left_of(&self, center_line: &Line, shape: &Bag<usize>) -> Vec<(Ortho, Color)> {
        let uncolored_orthos = self.uncolored_ortho_left(center_line, shape);

        uncolored_orthos
            .iter()
            .map(|o| (o.clone(), self.orthos[o].clone()))
            .collect()
    }

    fn line_left_of(&self, center_line: &Line) -> Vec<(Line, Color)> {
        let uncolored_lines = self.uncolored_line_left(center_line);

        uncolored_lines
            .iter()
            .map(|l| (l.clone(), self.lines[l].clone()))
            .collect()
    }

    fn line_right_of(&self, center_line: &Line) -> Vec<(Line, Color)> {
        let uncolored_lines = self.uncolored_line_right(center_line);

        uncolored_lines
            .iter()
            .map(|l| (l.clone(), self.lines[l].clone()))
            .collect()
    }

    fn ortho_right_of(&self, center_line: &Line, shape: &Bag<usize>) -> Vec<(Ortho, Color)> {
        let uncolored_orthos = self.uncolored_ortho_right(center_line, shape);

        uncolored_orthos
            .iter()
            .map(|o| (o.clone(), self.orthos[o].clone()))
            .collect()
    }

    fn uncolored_ortho_left(&self, center_line: &Line, shape: &Bag<usize>) -> Vec<Ortho> {
        self.source
            .square_left_of(center_line)
            .iter()
            .chain(&self.target.square_left_of(center_line))
            .filter(|o| &o.shape == shape)
            .cloned()
            .collect()
    }

    fn uncolored_line_left(&self, center_line: &Line) -> Vec<Line> {
        self.source
            .line_left_of(center_line)
            .iter()
            .chain(&self.target.line_left_of(center_line))
            .cloned()
            .collect()
    }

    fn uncolored_line_right(&self, center_line: &Line) -> Vec<Line> {
        self.source
            .line_right_of(center_line)
            .iter()
            .chain(&self.target.line_right_of(center_line))
            .cloned()
            .collect()
    }

    fn uncolored_ortho_right(&self, center_line: &Line, shape: &Bag<usize>) -> Vec<Ortho> {
        self.source
            .ortho_right_of(center_line)
            .iter()
            .chain(&self.target.ortho_right_of(center_line))
            .filter(|o| &o.shape == shape)
            .cloned()
            .collect()
    }

    fn match_by_discontinuity_black_o_l_o(
        &self,
        center_line: &Line,
        lhss: Vec<(Ortho, Color)>,
        rhss: Vec<(Ortho, Color)>,
    ) -> Vec<(Ortho, Line, Ortho)> {
        // | (Color::Red, Color::Black, Color::Black)
        // | (Color::Red, Color::Black, Color::Both)
        // | (Color::Red, Color::Black, Color::Red)
        // | (Color::Black, Color::Black, Color::Red)
        // | (Color::Both, Color::Black, Color::Red)

        find_discontinuity_with_color(lhss, rhss, center_line, Color::Red)
    }

    fn match_by_discontinuity_red_o_l_o(
        &self,
        center_line: &Line,
        lhss: Vec<(Ortho, Color)>,
        rhss: Vec<(Ortho, Color)>,
    ) -> Vec<(Ortho, Line, Ortho)> {
        // | (Color::Black, Color::Red, Color::Black)
        // | (Color::Black, Color::Red, Color::Red)
        // | (Color::Black, Color::Red, Color::Both)
        // | (Color::Red, Color::Red, Color::Black)
        // | (Color::Both, Color::Red, Color::Black)
        find_discontinuity_with_color(lhss, rhss, center_line, Color::Black)
    }

    fn match_by_discontinuity_both_o_l_o(
        &self,
        center_line: &Line,
        lhss: Vec<(Ortho, Color)>,
        rhss: Vec<(Ortho, Color)>,
    ) -> Vec<(Ortho, Line, Ortho)> {
        // | (Color::Black, Color::Both, Color::Red)
        // | (Color::Red, Color::Both, Color::Black)
        iproduct!(
            lhss.iter()
                .filter(|(_, color)| { color == &Color::Black })
                .map(|(o, _)| o.clone()),
            rhss.iter()
                .filter(|(_, color)| { color == &Color::Red })
                .map(|(o, _)| o.clone())
        )
        .map(|(l, r)| (l, center_line.to_owned(), r))
        .chain(
            iproduct!(
                lhss.iter()
                    .filter(|(_, color)| { color == &Color::Red })
                    .map(|(o, _)| o.clone()),
                rhss.iter()
                    .filter(|(_, color)| { color == &Color::Black })
                    .map(|(o, _)| o.clone())
            )
            .map(|(l, r)| (l, center_line.to_owned(), r)),
        )
        .collect_vec()
    }

    fn match_by_discontinuity_black_l_l_l(
        &self,
        center_line: &Line,
        lhss: Vec<(Line, Color)>,
        rhss: Vec<(Line, Color)>,
    ) -> Vec<(Line, Line, Line)> {
        // | (Color::Red, Color::Black, Color::Black)
        // | (Color::Red, Color::Black, Color::Both)
        // | (Color::Red, Color::Black, Color::Red)
        // | (Color::Black, Color::Black, Color::Red)
        // | (Color::Both, Color::Black, Color::Red)

        find_discontinuity_with_color_lll(lhss, rhss, center_line, Color::Red)
    }

    fn match_by_discontinuity_red_l_l_l(
        &self,
        center_line: &Line,
        lhss: Vec<(Line, Color)>,
        rhss: Vec<(Line, Color)>,
    ) -> Vec<(Line, Line, Line)> {
        // | (Color::Black, Color::Red, Color::Black)
        // | (Color::Black, Color::Red, Color::Red)
        // | (Color::Black, Color::Red, Color::Both)
        // | (Color::Red, Color::Red, Color::Black)
        // | (Color::Both, Color::Red, Color::Black)
        find_discontinuity_with_color_lll(lhss, rhss, center_line, Color::Black)
    }

    fn match_by_discontinuity_both_l_l_l(
        &self,
        center_line: &Line,
        lhss: Vec<(Line, Color)>,
        rhss: Vec<(Line, Color)>,
    ) -> Vec<(Line, Line, Line)> {
        // | (Color::Black, Color::Both, Color::Red)
        // | (Color::Red, Color::Both, Color::Black)
        iproduct!(
            lhss.iter()
                .filter(|(_, color)| { color == &Color::Black })
                .map(|(o, _)| o.clone()),
            rhss.iter()
                .filter(|(_, color)| { color == &Color::Red })
                .map(|(o, _)| o.clone())
        )
        .map(|(l, r)| (l, center_line.to_owned(), r))
        .chain(
            iproduct!(
                lhss.iter()
                    .filter(|(_, color)| { color == &Color::Red })
                    .map(|(o, _)| o.clone()),
                rhss.iter()
                    .filter(|(_, color)| { color == &Color::Black })
                    .map(|(o, _)| o.clone())
            )
            .map(|(l, r)| (l, center_line.to_owned(), r)),
        )
        .collect_vec()
    }
}

fn find_discontinuity_with_color_lll(lhss: Vec<(Line, Color)>, rhss: Vec<(Line, Color)>, center_line: &Line, other_color: Color) -> Vec<(Line, Line, Line)> {
    iproduct!(
        lhss.iter()
            .filter(|(_, color)| { color == &other_color })
            .map(|(o, _)| o.clone()),
        rhss.iter().map(|(o, _)| o.clone())
    )
    .map(|(l, r)| (l, center_line.to_owned(), r))
    .chain(
        iproduct!(
            lhss.iter()
                .filter(|(_, color)| { color != &other_color })
                .map(|(o, _)| o.clone()),
            rhss.iter()
                .filter(|(_, color)| { color == &other_color })
                .map(|(o, _)| o.clone())
        )
        .map(|(l, r)| (l, center_line.to_owned(), r)),
    )
    .collect_vec()
}

fn find_discontinuity_with_color(lhss: Vec<(Ortho, Color)>, rhss: Vec<(Ortho, Color)>, center_line: &Line, other_color: Color) -> Vec<(Ortho, Line, Ortho)> {
    iproduct!(
        lhss.iter()
            .filter(|(_, color)| { color == &other_color })
            .map(|(o, _)| o.clone()),
        rhss.iter().map(|(o, _)| o.clone())
    )
    .map(|(l, r)| (l, center_line.to_owned(), r))
    .chain(
        iproduct!(
            lhss.iter()
                .filter(|(_, color)| { color != &other_color })
                .map(|(o, _)| o.clone()),
            rhss.iter()
                .filter(|(_, color)| { color == &other_color })
                .map(|(o, _)| o.clone())
        )
        .map(|(l, r)| (l, center_line.to_owned(), r)),
    )
    .collect_vec()
}
