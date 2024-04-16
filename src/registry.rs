use std::collections::HashSet;

use crate::ortho::Ortho;

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
}
