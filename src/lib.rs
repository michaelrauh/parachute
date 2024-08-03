use std::{fs::read_to_string, time::Instant};

use ascii_table::{Align, AsciiTable};
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
mod hit_counter;
pub mod item;
pub mod line;
mod ortho;
mod registry;
mod s3_helper;
pub mod square;

#[tokio::main]
pub async fn add(file_name: String, endpoint: String, location: String) {
    let text = read_to_string(&file_name).unwrap();
    let bucket = Bucket::new(endpoint, location).await;

    if bucket.bucket_does_not_exist().await {
        bucket.create_bucket().await;
    }

    let body = read_file(&file_name);
    bucket.save_to_bucket_top_level(&file_name, body).await;

    let chunks: Vec<_> = text.split("CHAPTER").collect();
    let mut chunk_number = 0;
    for chunk_text in chunks {
        chunk_number += 1;
        let book_chunk = Book::book_from_text(&file_name, chunk_text, chunk_number);
        let registry = Registry::from_book(&book_chunk);
        bucket.write_chunk(registry).await;
    }
    bucket.delete_from_bucket_top_level(&file_name).await;
}

#[tokio::main]
pub async fn get(endpoint: String, location: String) {
    let bucket = Bucket::new(endpoint, location).await;

    bucket.dump_results().await;
}

#[tokio::main]
pub async fn delete(endpoint: String, location: String) {
    let bucket = Bucket::new(endpoint, location).await;

    bucket.delete_largest_answer().await
}

#[tokio::main]
pub async fn process(endpoint: String, location: String) {
    let bucket = Bucket::new(endpoint, location).await;
    loop {
        if let Some(registry) = bucket.checkout_smallest_chunk().await {
            let ans = single_process(&registry);

            let mut ascii_table = AsciiTable::default();
            ascii_table
                .column(0)
                .set_header("single")
                .set_align(Align::Left);

            let data: Vec<Vec<usize>> =
                vec![vec![ans.number_of_pairs()], vec![ans.number_of_squares()]];
            ascii_table.print(data);

            bucket.save_answer(ans).await;
            bucket.delete_chunk(registry).await;
        } else if let Some((source_answer, target_answer)) =
            bucket.checkout_largest_and_smallest_answer().await
        {
            let start = Instant::now();
            let new_answer = merge_process(&source_answer, &target_answer);
            let duration = start.elapsed();

            println!("Time elapsed in merge is: {:?}", duration);

            let mut ascii_table = AsciiTable::default();

            ascii_table
                .column(0)
                .set_header(source_answer.provenance.len().to_string())
                .set_align(Align::Left);
            ascii_table
                .column(1)
                .set_header(target_answer.provenance.len().to_string())
                .set_align(Align::Left);
            ascii_table
                .column(2)
                .set_header(new_answer.provenance.len().to_string())
                .set_align(Align::Left);

            let data: Vec<Vec<usize>> = vec![
                vec![
                    source_answer.number_of_pairs(),
                    target_answer.number_of_pairs(),
                    new_answer.number_of_pairs(),
                ],
                vec![
                    source_answer.number_of_squares(),
                    target_answer.number_of_squares(),
                    new_answer.number_of_squares(),
                ],
            ];
            ascii_table.print(data);
            bucket.save_answer(new_answer).await;
            bucket.delete_answer(source_answer).await;
            bucket.delete_answer(target_answer).await;
        } else {
            break;
        }
    }
}
