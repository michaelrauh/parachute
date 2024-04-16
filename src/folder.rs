use crate::{answer_helper::Answer, book_helper::book_helper::Book, ortho::Ortho, registry::Registry};

pub fn single_process(book: &Book) -> Answer {
    let new_squares = ffbb(book);
    let registry = Registry::new(new_squares);
    Answer::new(book.clone(), registry)
}

fn ffbb(book: &Book) -> Vec<Ortho> {
    let mut res = vec![];
    for (a, b) in book.pairs.clone() {
        for d in book.forward(b.clone()) {
            for c in book.backward(d.clone()) {
                if b != c && book.backward(c.clone()).contains(&a) {
                    res.push(Ortho::new(
                        a.to_string(),
                        b.to_string(),
                        c.clone(),
                        d.clone(),
                    ))
                }
            }
        }
    }
    res
}
