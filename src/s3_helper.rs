use aws_sdk_s3::{primitives::ByteStream, Client};
use futures::executor::block_on;

use crate::registry::Registry;

pub struct Bucket {
    client: Client,
    location: String,
}

impl Bucket {
    pub fn new(endpoint: String, location: String) -> Self {
        let client = aws_sdk_s3::Client::new(&block_on(
            aws_config::from_env().endpoint_url(endpoint).load(),
        ));
        Bucket { client, location }
    }

    pub fn bucket_does_not_exist(&self) -> bool {
        !block_on(self.client.list_buckets().send())
            .unwrap()
            .buckets()
            .to_vec()
            .iter()
            .any(|b| b.name().unwrap_or_default() == self.location)
    }

    pub fn delete_chunk(&self, registry: Registry) {
        self.delete_from_bucket_top_level(&("singleprocessing/".to_owned() + &registry.name))
    }

    pub fn save_answer(&self, ans: Registry) {
        let to_write = bincode::serialize(&ans).unwrap();
        let write_location = ans.name();

        self.save_to_bucket_top_level(&("answers/".to_string() + write_location), to_write.into());
    }

    pub fn delete_from_bucket_top_level(&self, file_name: &String) {
        block_on(
            self.client
                .delete_object()
                .bucket(self.location.clone())
                .key(file_name)
                .send(),
        )
        .unwrap();
    }

    pub fn write_chunk(&self, book_chunk: Registry) {
        let to_write = bincode::serialize(&book_chunk).unwrap();
        let write_location = book_chunk.name;

        self.save_to_bucket_top_level(&("chunks/".to_string() + &write_location), to_write.into());
    }

    pub fn checkout_smallest_chunk(&self) -> Option<Registry> {
        let f = self.get_smallest_file_name();

        if let Some(f) = f {
            self.move_chunk(&f);
            Some(self.read_chunk(&f))
        } else {
            None
        }
    }

    pub fn checkout_smallest_answer(&self) -> Option<Registry> {
        todo!()
    }

    pub fn checkout_largest_answer(&self) -> Option<Registry> {
        todo!()
    }

    fn read_chunk(&self, f: &str) -> Registry {
        let stream: ByteStream = block_on(
            self.client
                .get_object()
                .bucket(self.location.clone())
                .key("singleprocessing/".to_string() + f)
                .send(),
        )
        .unwrap()
        .body;

        let data = block_on(stream.collect())
            .expect("error reading data")
            .into_bytes();
        let b: Registry = bincode::deserialize(&data).unwrap();
        b
    }

    fn get_smallest_file_name(&self) -> Option<String> {
        let response = block_on(
            self.client
                .list_objects_v2()
                .bucket(self.location.clone())
                .send(),
        );

        let vec = response.unwrap().contents;
        let vec = &vec.unwrap();
        let minimum = vec
            .iter()
            .filter(|o| o.key().unwrap().split('/').next().unwrap().eq("chunks"))
            .min_by(|x, y| x.size.cmp(&y.size));

        if minimum.is_none() {
            None
        } else {
            let thing = minimum.unwrap();
            let key = &thing.key;
            let key = &key.clone().unwrap();
            let k = &key.split('/').last();
            Some((*k).unwrap().to_string())
        }
    }

    fn move_chunk(&self, file_name: &String) {
        let mut source_bucket_and_object = "".to_string();
        source_bucket_and_object.push_str(&self.location);
        source_bucket_and_object.push('/');
        source_bucket_and_object.push_str("chunks/");
        source_bucket_and_object.push_str(file_name);
        block_on(
            self.client
                .copy_object()
                .copy_source(source_bucket_and_object)
                .bucket(self.location.clone())
                .key("singleprocessing/".to_owned() + file_name)
                .send(),
        )
        .unwrap();
        self.delete_from_bucket_top_level(&("chunks/".to_owned() + file_name));
    }

    pub fn save_to_bucket_top_level(&self, file_name: &String, body: ByteStream) {
        block_on(
            self.client
                .put_object()
                .bucket(self.location.clone())
                .key(file_name)
                .body(body)
                .send(),
        )
        .unwrap();
    }

    pub fn create_bucket(&self) {
        block_on(
            self.client
                .create_bucket()
                .bucket(self.location.clone())
                .send(),
        )
        .unwrap();
    }
}
