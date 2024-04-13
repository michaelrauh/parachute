
pub mod s3_helper {
    use aws_sdk_s3::{primitives::ByteStream, Client};

    use crate::book_helper::book_helper::Book;

    pub async fn delete_from_bucket_top_level(client: &Client, location: &String, file_name: &String) {
        client
            .delete_object()
            .bucket(location)
            .key(file_name)
            .send()
            .await
            .unwrap();
    }

    pub async fn write_chunk(client: &Client, location: &String, book_chunk: Book) {
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

    pub async fn save_to_bucket_top_level(
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

    pub async fn create_bucket(client: &Client, location: &String) {
        client
            .create_bucket()
            .bucket(location)
            .send()
            .await
            .unwrap();
    }

    pub async fn bucket_does_not_exist(client: &Client, location: &String) -> bool {
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
}
