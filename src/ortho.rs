#[derive(PartialEq, Eq, Debug, Clone, Hash)]
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