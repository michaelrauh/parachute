use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Hash, Eq)]
pub struct Ortho {
    a: String,
    b: String,
    c: String,
    d: String,
}

impl Ortho {
    pub fn new(a: String, b: String, c: String, d: String) -> Self {
        Ortho { a, b, c, d }
    }
}
