pub mod book_helper {
    use serde::Serialize;

    pub fn book_from_text(file_name: &String, chunk: &str, chunk_number: usize) -> Book {
        let name = calculate_name(file_name, chunk_number);
        Book {
            name: name.clone(),
            provenance: vec![name],
            pairs: make_pairs(chunk),
        }
    }

    fn make_pairs(chunk: &str) -> Vec<(String, String)> {
        sentences_to_pairs(split_book_to_sentences(chunk.to_string()))
    }

    fn sentences_to_pairs(sentences: Vec<Vec<String>>) -> Vec<(String, String)> {
        sentences
            .iter()
            .filter(|sentence| sentence.len() > 1)
            .flat_map(|sentence| split_sentence_to_pairs(sentence.clone()))
            .collect()
    }

    fn split_sentence_to_pairs(words: Vec<String>) -> Vec<(String, String)> {
        let mut shifted = words.iter();
        shifted.next().expect("there must be something here");
        std::iter::zip(words.iter(), shifted)
            .map(|(f, s)| (f.clone(), s.clone()))
            .collect()
    }

    fn calculate_name(file_name: &str, chunk_number: usize) -> String {
        let (name, extension) = file_name.split_once(".").unwrap();
        name.to_string() + "-" + &chunk_number.to_string() + "." + extension
    }

    #[derive(Serialize, PartialEq, Debug)]
    pub struct Book {
        pub name: String,
        pub provenance: Vec<String>,
        pub pairs: Vec<(String, String)>,
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

    #[cfg(test)]
    mod tests {
        use crate::book_helper::book_helper::{calculate_name, make_pairs};

        #[test]
        fn test_make_pairs() {
            let result = make_pairs("This is a 5 chunk\n\nOf text that. has stops? end..");
            assert_eq!(
                result,
                vec![
                    ("this".to_string(), "is".to_string()),
                    ("is".to_string(), "a".to_string()),
                    ("a".to_string(), "chunk".to_string()),
                    ("of".to_string(), "text".to_string()),
                    ("text".to_string(), "that".to_string()),
                    ("has".to_string(), "stops".to_string())
                ]
            );
        }

        #[test]
        fn test_calculate_name() {
            let result = calculate_name("example.txt", 2);
            assert_eq!(result, "example-2.txt");
        }
    }
}
