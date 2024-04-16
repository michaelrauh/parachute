use std::fs::read_to_string;

use book_helper::book_from_text;
use file_helper::read_file;
use folder::single_process;
use s3_helper::{
    bucket_does_not_exist, checkout_smallest_chunk, create_bucket, delete_chunk,
    delete_from_bucket_top_level, save_to_bucket_top_level, write_chunk,
};
mod answer_helper;
mod book_helper;
mod file_helper;
mod folder;
mod ortho;
mod registry;
mod s3_helper;

#[::tokio::main]
pub async fn add(file_name: String, endpoint: String, location: String) {
    let config = aws_config::from_env().endpoint_url(endpoint).load().await; // todo move this into s3 helper and consider making it stateful to hide config / client
    let client = aws_sdk_s3::Client::new(&config);
    let text = read_to_string(&file_name).unwrap();

    if bucket_does_not_exist(&client, &location).await {
        // todo consider blocking on awaits. This is all sequential and fast compared to processing
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

#[::tokio::main]
pub async fn process(endpoint: String, location: String) {
    let config = aws_config::from_env().endpoint_url(endpoint).load().await;
    let client = aws_sdk_s3::Client::new(&config);
    let b = checkout_smallest_chunk(&client, &location).await.unwrap();
    let ans = single_process(&b);
    dbg!(ans.registry.squares.len()); // todo stop breaking law of demeter
    delete_chunk(&client, &location, b).await; // todo save the answer somewhere before doing multiple
}
