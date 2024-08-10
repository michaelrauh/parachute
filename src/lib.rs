use std::fs::read_to_string;

use charming::{
    element::{
        Emphasis, EmphasisFocus,
    },
    series::Sankey,
    Chart,
};

use book_helper::Book;
use file_helper::read_file;
use folder::{merge_process, single_process};
use itertools::Itertools;
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
    let mut names = vec![];
    let mut links = vec![];
    loop {
        if let Some(registry) = bucket.checkout_smallest_chunk().await {
            let ans: Registry = single_process(&registry);

            bucket.save_answer(ans.clone()).await;
            bucket.delete_chunk(registry).await;

            let count_by_shape = ans.count_by_shape();
            let new_links = count_by_shape.iter().map(|(shape, count)| {
                let mut shape_and_chunk = ans.name.clone();
                shape_and_chunk.push_str(&shape.iter().map(|x| x.to_string()).join(","));

                (ans.name.clone(), shape_and_chunk, *count as i32)
            });

            for link in new_links {
                links.push(link.clone());

                if !names.contains(&link.0) {
                    names.push(link.0);
                }
                
                if !names.contains(&link.1) {
                    names.push(link.1);
                }
            }

            dbg!(&names, &links);
            display_report(&names, &links);
        } else if let Some((source_answer, target_answer)) =
            bucket.checkout_largest_and_smallest_answer().await
        {
            let new_answer = merge_process(&source_answer, &target_answer);
            
            display_report(&names, &links);
            bucket.save_answer(new_answer).await;
            bucket.delete_answer(source_answer).await;
            bucket.delete_answer(target_answer).await;
        } else {
            break;
        }
    }
}

fn display_report(names: &Vec<String>, links: &Vec<(String, String, i32)>) {
    let chart = Chart::new().series(
        Sankey::new()
            .emphasis(Emphasis::new().focus(EmphasisFocus::Adjacency))
            .data(names.to_owned())
            .links(links.to_owned()),
    );
    let mut renderer = charming::ImageRenderer::new(1000, 800);
    renderer.save(&chart, "sankey.svg").unwrap();
}
