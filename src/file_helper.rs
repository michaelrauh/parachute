use std::path::Path;

use aws_sdk_s3::primitives::ByteStream;

pub async fn read_file(file_name: &String) -> ByteStream {
    ByteStream::from_path(Path::new(&file_name)).await.unwrap()
}
