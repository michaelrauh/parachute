use std::fs::read_to_string;

use book_helper::book_from_text;
use file_helper::read_file;
use folder::single_process;
use s3_helper::Bucket;


mod answer_helper;
mod book_helper;
mod file_helper;
mod folder;
mod ortho;
mod registry;
mod s3_helper;

#[::tokio::main]
pub async fn add(file_name: String, endpoint: String, location: String) {
    let text = read_to_string(&file_name).unwrap();
    let bucket = Bucket::new(endpoint, location).await;

    if bucket.bucket_does_not_exist().await {
        // todo consider blocking on awaits. This is all sequential and fast compared to processing
        bucket.create_bucket().await;
    }

    let body = read_file(&file_name).await;
    bucket.save_to_bucket_top_level(&file_name, body).await;

    let chunks: Vec<_> = text.split("CHAPTER").collect();
    let mut chunk_number = 0;
    for chunk_text in chunks {
        chunk_number += 1;
        let book_chunk = book_from_text(&file_name, chunk_text, chunk_number);
        bucket.write_chunk(book_chunk).await;
    }
    bucket.delete_from_bucket_top_level(&file_name).await;
}

#[::tokio::main]
pub async fn process(endpoint: String, location: String) {
    let bucket = Bucket::new(endpoint, location).await;
    let b = bucket.checkout_smallest_chunk().await.unwrap();
    let ans = single_process(&b);
    dbg!(ans.size());
    bucket.save_answer(ans).await;
    bucket.delete_chunk(b).await;
}
