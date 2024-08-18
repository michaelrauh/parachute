use crate::line::Line;
use crate::ortho::Ortho;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Item {
    Pair(Line),
    Square(Ortho),
}
