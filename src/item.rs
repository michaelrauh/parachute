use crate::line::Line;
use crate::ortho::Ortho;

#[derive(PartialEq, Debug, Clone)]
pub enum Item<'a> {
    Pair(&'a Line),
    Square(&'a Ortho),
}
