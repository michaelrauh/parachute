use serde::{Deserialize, Serialize};
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Eq, Hash)]
pub struct Line {
    pub first: String,
    pub second: String,
}

impl From<Line> for (String, String) {
    fn from(val: Line) -> Self {
        (val.first, val.second)
    }
}
