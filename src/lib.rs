use std::fs::read_to_string;

use book_helper::Book;
use file_helper::read_file;
use folder::{merge_process, single_process};
use s3_helper::Bucket;

use crate::registry::Registry;

mod book_helper;
pub mod color;
pub mod discontinuity_detector;
mod file_helper;
mod folder;
pub mod item;
pub mod line;
mod ortho;
mod registry;
mod s3_helper;

#[tokio::main]
pub async fn add(file_name: String, endpoint: String, location: String) {
    let text = read_to_string(&file_name).unwrap();
    let bucket = Bucket::new(endpoint, location);

    if bucket.bucket_does_not_exist() {
        bucket.create_bucket();
    }

    let body = read_file(&file_name);
    bucket.save_to_bucket_top_level(&file_name, body);

    let chunks: Vec<_> = text.split("CHAPTER").collect();
    let mut chunk_number = 0;
    for chunk_text in chunks {
        chunk_number += 1;
        let book_chunk = Book::book_from_text(&file_name, chunk_text, chunk_number);
        let registry = Registry::from_book(&book_chunk);
        bucket.write_chunk(registry);
    }
    bucket.delete_from_bucket_top_level(&file_name);
}

#[tokio::main]
pub async fn process(endpoint: String, location: String) {
    let bucket = Bucket::new(endpoint, location);

    if let Some(registry) = bucket.checkout_smallest_chunk() {
        let ans = single_process(&registry);
        dbg!(ans.size());
        bucket.save_answer(ans);
        bucket.delete_chunk(registry);
    } else {
        if let Some((source_answer, target_answer)) = bucket.checkout_largest_and_smallest_answer() {
            let new_answer = merge_process(&source_answer, &target_answer);
            bucket.save_answer(new_answer);
            bucket.delete_answer(source_answer);
            bucket.delete_answer(target_answer);
        } else {
            return;
        }
    }
}
