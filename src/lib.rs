use aws_sdk_s3::{primitives::ByteStream, Client};
use serde::Serialize;
use std::{fs::read_to_string, path::Path};

// todo split this file out
#[::tokio::main]
pub async fn add(file_name: String, endpoint: String, location: String) {
    let config = aws_config::from_env().endpoint_url(endpoint).load().await;
    let client = aws_sdk_s3::Client::new(&config);
    let text = read_to_string(&file_name).unwrap();

    if bucket_does_not_exist(&client, &location).await {
        create_bucket(&client, &location).await;
    }

    let body = read_file(&file_name).await;
    save_to_bucket_top_level(&client, &location, &file_name, body).await;

    let chunks: Vec<_> = text.split("CHAPTER").collect(); // todo: pass in pattern from command line
    let mut chunk_number = 0;
    for chunk_text in chunks {
        chunk_number += 1;
        let book_chunk = book_from_text(&file_name, chunk_text, chunk_number);
        write_chunk(&client, &location, book_chunk).await;
    }
    delete_from_bucket_top_level(&client, &location, &file_name).await;
}

async fn delete_from_bucket_top_level(client: &Client, location: &String, file_name: &String) {
    client
        .delete_object()
        .bucket(location)
        .key(file_name)
        .send()
        .await
        .unwrap();
}

async fn write_chunk(client: &Client, location: &String, book_chunk: Book) {
    let to_write = bincode::serialize(&book_chunk).unwrap();
    let write_location = book_chunk.name;

    save_to_bucket_top_level(
        &client,
        &location,
        &("chunks/".to_string() + &write_location),
        to_write.into(),
    )
    .await;
}

fn book_from_text(file_name: &String, chunk: &str, chunk_number: usize) -> Book {
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
struct Book {
    name: String,
    provenance: Vec<String>,
    pairs: Vec<(String, String)>,
}

async fn save_to_bucket_top_level(
    client: &Client,
    location: &String,
    file_name: &String,
    body: ByteStream,
) {
    client
        .put_object()
        .bucket(location)
        .key(file_name)
        .body(body)
        .send()
        .await
        .unwrap();
}

async fn read_file(file_name: &String) -> ByteStream {
    ByteStream::from_path(Path::new(&file_name)).await.unwrap()
}

async fn create_bucket(client: &Client, location: &String) {
    client
        .create_bucket()
        .bucket(location)
        .send()
        .await
        .unwrap();
}

async fn bucket_does_not_exist(client: &Client, location: &String) -> bool {
    !client
        .list_buckets()
        .send()
        .await
        .unwrap()
        .buckets()
        .to_vec()
        .iter()
        .any(|b| b.name().unwrap_or_default() == location)
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
                }).filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{calculate_name, make_pairs};

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
