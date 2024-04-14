pub mod s3_helper {
    use aws_sdk_s3::{
        primitives::ByteStream,
        Client,
    };

    use crate::book_helper::book_helper::Book;

    pub async fn delete_from_bucket_top_level(
        client: &Client,
        location: &String,
        file_name: &String,
    ) {
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

    pub async fn checkout_smallest_chunk(client: &Client, bucket: &str) -> Option<Book> {
        let f = get_smallest_file_name(client, bucket).await;

        if f.is_none() {
            return None;
        } else {
            let f = &f.unwrap();
            move_chunk(client, &bucket.to_string(), f).await;
            Some(read_chunk(client, bucket, f).await)
        }
    }

    async fn read_chunk(client: &Client, bucket: &str, f: &str) -> Book {


        let stream: ByteStream = client
            .get_object()
            .bucket(bucket)
            .key("singleprocessing/".to_string() + f)
            .send()
            .await
            .unwrap()
            .body;

        let data = stream
            .collect()
            .await
            .expect("error reading data")
            .into_bytes();
        let b: Book = bincode::deserialize(&data).unwrap();
        b
    }

    async fn get_smallest_file_name(client: &Client, bucket: &str) -> Option<String> {
        let response = client
            .list_objects_v2()
            .bucket(bucket.to_owned())
            .send()
            .await;

        let vec = response.unwrap().contents;
        let vec = &vec.unwrap();
        let minimum = vec.iter().filter(|o| o.key().unwrap().split("/").next().unwrap().eq("chunks")).min_by(|x, y| x.size.cmp(&y.size));

        if minimum.is_none() {
            return None;
        } else {
            let thing = minimum.unwrap();
            let key = &thing.key;
            let key = &key.clone().unwrap();
            let k = &key.split("/").last();
            return Some(k.clone().unwrap().to_string());
        }
    }

    async fn move_chunk(client: &Client, location: &String, file_name: &String) {
        let mut source_bucket_and_object = "".to_string();
        source_bucket_and_object.push_str(&location);
        source_bucket_and_object.push('/');
        source_bucket_and_object.push_str("chunks/");
        source_bucket_and_object.push_str(&file_name);
        client
            .copy_object()
            .copy_source(source_bucket_and_object)
            .bucket(location)
            .key("singleprocessing/".to_owned() + file_name)
            .send()
            .await
            .unwrap();
        delete_from_bucket_top_level(client, location, &("chunks/".to_owned() + file_name)).await;
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
