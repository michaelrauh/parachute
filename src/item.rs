use crate::line::Line;
use crate::ortho::Ortho;

#[derive(PartialEq, Debug, Clone)]
pub enum Item {
    Pair(Line),
    Square(Ortho),
}
