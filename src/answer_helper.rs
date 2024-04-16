use crate::{book_helper::Book, registry::Registry};

pub struct Answer {
    pub book: Book,
    pub registry: Registry,
}
impl Answer {
    pub(crate) fn new(book: Book, registry: Registry) -> Answer {
        Answer { book, registry }
    }
}
