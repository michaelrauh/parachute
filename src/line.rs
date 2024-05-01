use serde::{Deserialize, Serialize};
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Eq, Hash)]
pub struct Line {
    pub first: String,
    pub second: String,
}

impl Into<(String, String)> for Line {
    fn into(self) -> (String, String) {
        (self.first, self.second)
    }
}
