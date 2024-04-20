use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::ortho::Ortho;
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Registry {
    // todo consider hiding this behind s3 helper or vice-versa. Alternatively consider the same for the answer struct
    pub squares: HashSet<Ortho>,
}
impl Registry {
    pub(crate) fn new(new_squares: Vec<Ortho>) -> Self {
        Registry {
            squares: HashSet::from_iter(new_squares),
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.squares.len()
    }
}
