use std::collections::HashSet;

use crate::line::Line;

fn sentences_to_pairs(sentences: Vec<Vec<String>>) -> Vec<Line> {
    sentences
        .iter()
        .filter(|sentence| sentence.len() > 1)
        .flat_map(|sentence| split_sentence_to_pairs(sentence.clone()))
        .collect()
}

fn split_sentence_to_pairs(words: Vec<String>) -> Vec<Line> {
    let mut shifted = words.iter();
    shifted.next().expect("there must be something here");
    std::iter::zip(words.iter(), shifted)
        .map(|(f, s)| Line {
            first: f.clone(),
            second: s.clone(),
        })
        .collect()
}

#[derive(PartialEq, Debug, Clone)]
pub struct Book {
    chunk: String,
    file_name: String,
    chunk_number: usize,
}

impl Book {
    pub fn book_from_text(file_name: &str, chunk: &str, chunk_number: usize) -> Self {
        Book {
            chunk: chunk.to_owned(),
            file_name: file_name.to_owned(),
            chunk_number,
        }
    }
    pub fn make_pairs(&self) -> HashSet<Line> {
        HashSet::from_iter(sentences_to_pairs(split_book_to_sentences(
            self.chunk.to_string(),
        )))
    }
    pub fn calculate_name(&self) -> String {
        let (name, extension) = self.file_name.split_once('.').unwrap();
        name.to_string() + "-" + &self.chunk_number.to_string() + "." + extension
    }
}

pub fn split_book_to_sentences(book: String) -> Vec<Vec<String>> {
    book.split_terminator(&['.', '!', '?', ';', '\n'])
        .filter(|x| !x.is_empty())
        .map(|x| x.trim())
        .map(|sentence| {
            sentence
                .split_ascii_whitespace()
                .map(|s| {
                    s.chars()
                        .filter(|c| c.is_alphabetic())
                        .collect::<String>()
                        .to_lowercase()
                })
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .collect()
}
