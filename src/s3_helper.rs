use aws_sdk_s3::{primitives::ByteStream, Client};
use itertools::Itertools;

use crate::registry::Registry;

pub struct Bucket {
    client: Client,
    location: String,
}

impl Bucket {
    pub async fn new(endpoint: String, location: String) -> Self {
        let client =
            aws_sdk_s3::Client::new(&aws_config::from_env().endpoint_url(endpoint).load().await);
        Bucket { client, location }
    }

    pub async fn bucket_does_not_exist(&self) -> bool {
        !self
            .client
            .list_buckets()
            .send()
            .await
            .unwrap()
            .buckets()
            .to_vec()
            .iter()
            .any(|b| b.name().unwrap_or_default() == self.location)
    }

    pub async fn delete_chunk(&self, registry: &Registry) {
        self.delete_from_bucket_top_level(&("singleprocessing/".to_owned() + &registry.name))
            .await
    }

    pub async fn delete_answer(&self, registry: &Registry) {
        self.delete_from_bucket_top_level(&("doubleprocessing/".to_owned() + &registry.name))
            .await
    }

    pub async fn delete_largest_answer(&self) {
        let f = self.get_largest_file_name("answers").await;
        self.delete_from_bucket_top_level(&("answers/".to_owned() + &f.unwrap()))
            .await
    }

    pub async fn save_answer(&self, ans: &Registry) {
        let to_write = bincode::serialize(&ans).unwrap();
        let write_location = ans.name();

        self.save_to_bucket_top_level(&("answers/".to_string() + write_location), to_write.into())
            .await;
    }

    pub async fn delete_from_bucket_top_level(&self, file_name: &String) {
        self.client
            .delete_object()
            .bucket(self.location.clone())
            .key(file_name)
            .send()
            .await
            .unwrap();
    }

    pub async fn dump_results(&self) {
        // dbg!(self.read_largest_chunk().await.unwrap().squares);
    }

    pub async fn write_chunk(&self, book_chunk: Registry) {
        let to_write = bincode::serialize(&book_chunk).unwrap();
        let write_location = book_chunk.name;

        self.save_to_bucket_top_level(&("chunks/".to_string() + &write_location), to_write.into())
            .await;
    }

    pub async fn checkout_smallest_chunk(&self) -> Option<Registry> {
        let f = self.get_smallest_file_name("chunks").await;

        if let Some(f) = f {
            self.move_chunk(&f, "chunks", "singleprocessing/").await;
            Some(self.read_chunk(&f, "singleprocessing/").await)
        } else {
            None
        }
    }

    pub async fn checkout_largest_and_smallest_answer(&self) -> Option<(Registry, Registry)> {
        let f = self.get_largest_and_smallest_file_name("answers").await;

        if let Some((l, s)) = f {
            self.move_chunk(&l, "answers", "doubleprocessing/").await;
            self.move_chunk(&s, "answers", "doubleprocessing/").await;
            Some((
                self.read_chunk(&l, "doubleprocessing/").await,
                self.read_chunk(&s, "doubleprocessing/").await,
            ))
        } else {
            None
        }
    }
    #[allow(dead_code)]
    pub async fn read_largest_chunk(&self) -> Option<Registry> {
        let f = self.get_largest_file_name("answers").await;

        if let Some(l) = f {
            Some(self.read_chunk(&l, "answers/").await)
        } else {
            None
        }
    }

    async fn read_chunk(&self, f: &str, prefix: &str) -> Registry {
        let stream: ByteStream = self
            .client
            .get_object()
            .bucket(self.location.clone())
            .key(prefix.to_string() + f)
            .send()
            .await
            .unwrap()
            .body;

        let data = stream
            .collect()
            .await
            .expect("error reading data")
            .into_bytes();
        let b: Registry = bincode::deserialize(&data).unwrap();
        b
    }

    async fn get_smallest_file_name(&self, prefix: &str) -> Option<String> {
        let response = self
            .client
            .list_objects_v2()
            .bucket(self.location.clone())
            .send()
            .await;

        let vec = response.unwrap().contents;
        let vec = &vec.unwrap();
        let minimum = vec
            .iter()
            .filter(|o| o.key().unwrap().split('/').next().unwrap().eq(prefix))
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

    async fn get_largest_file_name(&self, prefix: &str) -> Option<String> {
        let response = self
            .client
            .list_objects_v2()
            .bucket(self.location.clone())
            .send()
            .await;

        let vec = response.unwrap().contents;
        let vec = &vec.unwrap();
        let maximum = vec
            .iter()
            .filter(|o| o.key().unwrap().split('/').next().unwrap().eq(prefix))
            .max_by(|x, y| x.size.cmp(&y.size));

        if maximum.is_none() {
            None
        } else {
            let thing = maximum.unwrap();
            let key = &thing.key;
            let key = &key.clone().unwrap();
            let k = &key.split('/').last();
            Some((*k).unwrap().to_string())
        }
    }

    async fn move_chunk(&self, file_name: &String, prefix: &str, processing_prefix: &str) {
        let mut source_bucket_and_object = "".to_string();
        source_bucket_and_object.push_str(&self.location);
        source_bucket_and_object.push('/');
        source_bucket_and_object.push_str(prefix);
        source_bucket_and_object.push('/');
        source_bucket_and_object.push_str(file_name);

        self.client
            .copy_object()
            .copy_source(source_bucket_and_object)
            .bucket(self.location.clone())
            .key(processing_prefix.to_owned() + file_name)
            .send()
            .await
            .unwrap();
        self.delete_from_bucket_top_level(&(prefix.to_owned() + "/" + file_name))
            .await;
    }

    pub async fn save_to_bucket_top_level(&self, file_name: &String, body: ByteStream) {
        self.client
            .put_object()
            .bucket(self.location.clone())
            .key(file_name)
            .body(body)
            .send()
            .await
            .unwrap();
    }

    pub async fn create_bucket(&self) {
        self.client
            .create_bucket()
            .bucket(self.location.clone())
            .send()
            .await
            .unwrap();
    }

    async fn get_largest_and_smallest_file_name(&self, prefix: &str) -> Option<(String, String)> {
        let response = self
            .client
            .list_objects_v2()
            .bucket(self.location.clone())
            .send()
            .await;

        let vec = response.unwrap().contents;
        let vec = &vec.unwrap();
        let maximum = vec
            .iter()
            .filter(|o| o.key().unwrap().split('/').next().unwrap().eq(prefix))
            .minmax_by(|x, y| x.size.cmp(&y.size));

        match maximum {
            itertools::MinMaxResult::NoElements => None,
            itertools::MinMaxResult::OneElement(_) => None,
            itertools::MinMaxResult::MinMax(min, max) => {
                Some((extract_filename(min), extract_filename(max)))
            }
        }
    }
}

fn extract_filename(min: &aws_sdk_s3::types::Object) -> String {
    let thing = min;
    let key = &thing.key;
    let key = &key.clone().unwrap();
    let k = &key.split('/').last();
    (*k).unwrap().to_string()
}
