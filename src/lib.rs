use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
};

use bag::Bag;
use book_helper::Book;
use file_helper::read_file;
use folder::{merge_process, single_process};
use itertools::Itertools;
use s3_helper::Bucket;

use crate::registry::Registry;

pub mod bag;
mod book_helper;
pub mod color;
pub mod discontinuity_detector;
mod file_helper;
pub mod folder;
pub mod line;
mod ortho;
pub mod registry;
mod s3_helper;

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
        if let Some(mut registry) = bucket.checkout_smallest_chunk().await {
            dbg!(&registry.name);
            single_process(&mut registry);

            bucket.save_answer(&registry).await;
            bucket.delete_chunk(&registry).await;

            for (shape, count) in registry.count_by_shape() {
                let print_shape = shape.iter().join(",");
                println!("{:<15}: {:>5}", print_shape, count.to_string());
            }
        } else if let Some((mut source_answer, target_answer)) =
            bucket.checkout_largest_and_smallest_answer().await
        {
            dbg!(&source_answer.name, &target_answer.name);
            let all_shapes: HashSet<Bag<usize>> = source_answer
                .count_by_shape()
                .map(|(s, _c)| s)
                .chain(target_answer.count_by_shape().map(|(s, _c)| s))
                .cloned()
                .collect();
            for shape in all_shapes.clone() {
                let source_count = source_answer
                    .count_by_shape()
                    .find(|(s, _c)| *s == &shape)
                    .map(|(_s, c)| c)
                    .unwrap_or_default();
                let target_count = target_answer
                    .count_by_shape()
                    .find(|(s, _c)| *s == &shape)
                    .map(|(_s, c)| c)
                    .unwrap_or_default();
                let print_shape = shape.iter().join(",");
                println!(
                    "{:<15}: {:>5} + {:>5} = ",
                    print_shape, source_count, target_count
                );
            }
            let mut source_counts = HashMap::new();
            for shape in all_shapes {
                let source_count = source_answer
                    .count_by_shape()
                    .find(|(s, _c)| *s == &shape)
                    .map(|(_s, c)| c)
                    .unwrap_or_default();
                source_counts.insert(shape, source_count);
            }

            merge_process(&mut source_answer, &target_answer);

            bucket.save_answer(&source_answer).await;
            bucket.delete_answer(&source_answer).await;
            bucket.delete_answer(&target_answer).await;

            let all_shapes: HashSet<Bag<usize>> = source_answer
                .count_by_shape()
                .map(|(s, _c)| s)
                .chain(target_answer.count_by_shape().map(|(s, _c)| s))
                .chain(source_answer.count_by_shape().map(|(s, _c)| s))
                .cloned()
                .collect();
            for shape in all_shapes {
                let target_count = target_answer
                    .count_by_shape()
                    .find(|(s, _c)| *s == &shape)
                    .map(|(_s, c)| c)
                    .unwrap_or_default();
                let new_count = source_answer
                    .count_by_shape()
                    .find(|(s, _c)| *s == &shape)
                    .map(|(_s, c)| c)
                    .unwrap_or_default();
                let discovered =
                    new_count - (source_counts.get(&shape).unwrap_or(&0) + target_count);
                let print_shape = shape.iter().join(",");
                let equation = format!(
                    "{:>5} + {:>5} = {:>5}",
                    source_counts.get(&shape).unwrap_or(&0),
                    target_count,
                    new_count
                );
                println!(
                    "{:<15}: {:<25} ({:>2} new)",
                    print_shape, equation, discovered
                );
            }
        } else {
            break;
        }
    }
}
