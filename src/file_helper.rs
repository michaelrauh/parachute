use std::path::Path;

use aws_sdk_s3::primitives::ByteStream;
use futures::executor::block_on;

pub fn read_file(file_name: &String) -> ByteStream {
    block_on(ByteStream::from_path(Path::new(&file_name))).unwrap()
}
