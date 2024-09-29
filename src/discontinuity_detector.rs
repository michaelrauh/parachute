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

    pub(crate) fn o_l_o_discontinuities<'a>(&'a self) -> Vec<(&'a Ortho, &'a Line, &'a Ortho)> {
        let centers = self.centers();
    
        let left_shapes = self
            .source
            .count_by_shape()
            .map(|(s, _c)| s)
            .collect_vec();
    
        let right_shapes = self
            .target
            .count_by_shape()
            .map(|(s, _c)| s)
            .collect_vec();
    
        let shapes = left_shapes
            .into_iter()
            .filter(move |s| right_shapes.contains(s));
    
        shapes
            .flat_map(move |shape| {
                let centers = centers.clone();
                centers.iter().flat_map(move |(center_line, center_color)| {
                    let lhss = self.ortho_left_of(center_line, shape);
                    let rhss = self.ortho_right_of(center_line, shape);
    
                    match center_color {
                        Color::Black => self.match_by_discontinuity_black_o_l_o(center_line, lhss, rhss),
                        Color::Red => self.match_by_discontinuity_red_o_l_o(center_line, lhss, rhss),
                        Color::Both => self.match_by_discontinuity_both_o_l_o(center_line, lhss, rhss),
                    }
                }).collect::<Vec<_>>()
            })
            .filter(|(l, _c, r)| l.valid_diagonal_with(r))
            .collect()
    }
    
    fn centers<'a>(&'a self) -> Vec<(&'a Line, &'a Color)> {
        self.lines.iter().collect()
    }    

    fn ortho_left_of<'a>(&'a self, center_line: &Line, shape: &'a Bag<usize>) -> impl Iterator<Item = (&'a Ortho, &'a Color)> {
        let uncolored_orthos = self.uncolored_ortho_left(center_line, shape);

        uncolored_orthos
            .into_iter()
            .map(|o| (o, &self.orthos[o]))
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

    fn ortho_right_of<'a>(&'a self, center_line: &Line, shape: &'a Bag<usize>) -> impl Iterator<Item = (&'a Ortho, &'a Color)> {
        let uncolored_orthos = self.uncolored_ortho_right(center_line, shape);

        uncolored_orthos
            .map(move |o| (o, &self.orthos[o]))
    }

    fn uncolored_ortho_left<'a>(&'a self, center_line: &Line, shape: &'a Bag<usize>) -> impl Iterator<Item = &'a Ortho> + 'a {
        self.source
            .square_left_of(center_line)
            .chain(self.target.square_left_of(center_line))
            .filter(move |o| &o.shape == shape)
    }

    fn uncolored_line_left(&self, center_line: &Line) -> Vec<Line> {
        self.source
            .line_left_of(center_line)
            .chain(self.target.line_left_of(center_line))
            .cloned()
            .collect()
    }

    fn uncolored_line_right(&self, center_line: &Line) -> Vec<Line> {
        self.source
            .line_right_of(center_line)
            .chain(self.target.line_right_of(center_line))
            .cloned()
            .collect()
    }

    fn uncolored_ortho_right<'a>(
        &'a self,
        center_line: &Line,
        shape: &'a Bag<usize>,
    ) -> impl Iterator<Item = &'a Ortho> + 'a {
        self.source
            .ortho_right_of(center_line)
            .chain(self.target.ortho_right_of(center_line))
            .filter(move |o| &o.shape == shape)
    }
    
    fn match_by_discontinuity_black_o_l_o<'a>(
        &self,
        center_line: &'a Line,
        lhss: impl Iterator<Item = (&'a Ortho, &'a Color)> + 'a,
        rhss: impl Iterator<Item = (&'a Ortho, &'a Color)> + 'a,
    ) -> Vec<(&'a Ortho, &'a Line, &'a Ortho)> {
        // | (Color::Red, Color::Black, Color::Black)
        // | (Color::Red, Color::Black, Color::Both)
        // | (Color::Red, Color::Black, Color::Red)
        // | (Color::Black, Color::Black, Color::Red)
        // | (Color::Both, Color::Black, Color::Red)

        find_discontinuity_with_color(lhss, rhss, center_line, Color::Red)
    }

    fn match_by_discontinuity_red_o_l_o<'a>(
        &self,
        center_line: &'a Line,
        lhss: impl Iterator<Item = (&'a Ortho, &'a Color)> + 'a,
        rhss: impl Iterator<Item = (&'a Ortho, &'a Color)> + 'a,
    ) -> Vec<(&'a Ortho, &'a Line, &'a Ortho)> {
        // | (Color::Black, Color::Red, Color::Black)
        // | (Color::Black, Color::Red, Color::Red)
        // | (Color::Black, Color::Red, Color::Both)
        // | (Color::Red, Color::Red, Color::Black)
        // | (Color::Both, Color::Red, Color::Black)
        find_discontinuity_with_color(lhss, rhss, center_line, Color::Black)
    }

    fn match_by_discontinuity_both_o_l_o<'a>(
        &self,
        center_line: &'a Line,
        lhss: impl Iterator<Item = (&'a Ortho, &'a Color)> + 'a,
        rhss: impl Iterator<Item = (&'a Ortho, &'a Color)> + 'a,
    ) -> Vec<(&'a Ortho, &'a Line, &'a Ortho)> {
        // | (Color::Black, Color::Both, Color::Red)
        // | (Color::Red, Color::Both, Color::Black)

        let lhss = lhss.collect_vec();
        let rhss = rhss.collect_vec();
        let firsts = iproduct!(
            lhss.iter()
                .filter(|(_, color)| { color == &&Color::Black })
                .map(|(o, _)| o),
            rhss.iter()
                .filter(|(_, color)| { **color == Color::Red })
                .map(|(o, _)| o)
        )
        .map(|(l, r)| (l, center_line, r));

        let seconds = 
            iproduct!(
                lhss.iter()
                    .filter(|(_, color)| { color == &&Color::Red })
                    .map(|(o, _)| o),
                rhss.iter()
                    .filter(|(_, color)| { color == &&Color::Black })
                    .map(|(o, _)| o)
            )
            .map(|(l, r)| (l, center_line, r));
        firsts.chain(seconds).map(|(l, c, r)| (*l, c, *r)).collect_vec()
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

fn find_discontinuity_with_color_lll(
    lhss: Vec<(Line, Color)>,
    rhss: Vec<(Line, Color)>,
    center_line: &Line,
    other_color: Color,
) -> Vec<(Line, Line, Line)> {
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

fn find_discontinuity_with_color<'a>(
    lhss: impl Iterator<Item = (&'a Ortho, &'a Color)> + 'a,
    rhss: impl Iterator<Item = (&'a Ortho, &'a Color)> + 'a,
    center_line: &'a Line,
    other_color: Color,
) -> Vec<(&'a Ortho, &'a Line, &'a Ortho)> {
    let lhss: Vec<(&Ortho, &Color)> = lhss.collect_vec();
    let rhss: Vec<(&Ortho, &Color)> = rhss.collect_vec();
    iproduct!(
        lhss.iter()
            .filter(|(_, color)| { color == &&other_color })
            .map(|(o, _)| o),
        rhss.iter().map(|(o, _)| o)
    )
    .map(|(l, r)| (*l, center_line, *r))
    .chain(
        iproduct!(
            lhss.iter()
                .filter(|(_, color)| { color != &&other_color })
                .map(|(o, _)| o),
            rhss.iter()
                .filter(|(_, color)| { color == &&other_color })
                .map(|(o, _)| o)
        )
        .map(|(l, r)| (*l, center_line, *r)),
    )
    .collect_vec()
}
