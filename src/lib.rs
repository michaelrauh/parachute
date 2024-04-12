use std::path::Path;

use aws_sdk_s3::{primitives::ByteStream, Client};

#[::tokio::main]
pub async fn add(file_name: String, endpoint: String, location: String) {
    let config = aws_config::from_env().endpoint_url(endpoint).load().await;
    let client = aws_sdk_s3::Client::new(&config);

    if bucket_does_not_exist(&client, &location).await
    {
        create_bucket(&client, &location).await;
    }

    let body = read_file(&file_name).await;
    save_to_bucket_top_level(&client, &location, &file_name, body).await;
}

async fn save_to_bucket_top_level(client: &Client, location: &String, file_name: &String, body: ByteStream) {
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
