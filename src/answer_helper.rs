use serde::{Deserialize, Serialize};

use crate::{book_helper::Book, registry::Registry};
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Answer {
    pub book: Book,
    pub registry: Registry,
}
impl Answer {
    pub(crate) fn new(book: Book, registry: Registry) -> Answer {
        Answer { book, registry }
    }

    pub fn name(&self) -> &str {
        &self.book.name
    }

    pub fn size(&self) -> usize {
        self.registry.size()
    }
}
