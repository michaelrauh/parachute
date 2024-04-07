use std::path::Path;

use aws_sdk_s3::primitives::ByteStream;

#[::tokio::main]
pub async fn add(file_name: String, endpoint: String, location: String) {
    let config = aws_config::from_env().endpoint_url(endpoint).load().await;
    let client = aws_sdk_s3::Client::new(&config);

    if !client
        .list_buckets()
        .send()
        .await
        .unwrap()
        .buckets()
        .to_vec()
        .iter()
        .any(|b| b.name().unwrap_or_default() == location)
    {
        client
            .create_bucket()
            .bucket(&location)
            .send()
            .await
            .unwrap();
    }

    let body = ByteStream::from_path(Path::new(&file_name)).await;
    client
        .put_object()
        .bucket(location)
        .key(file_name)
        .body(body.unwrap())
        .send()
        .await
        .unwrap();
}
