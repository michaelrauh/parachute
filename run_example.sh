set -ex

aws --endpoint-url http://192.168.1.9:9000 s3api create-bucket --bucket database

cargo run -- --add example.txt --location database --endpoint http://192.168.1.9:9000
aws --endpoint-url http://192.168.1.9:9000 s3 ls database/
aws --endpoint-url http://192.168.1.9:9000 s3 cp s3://database/example.txt example-result.txt

diff example.txt example-result.txt

rm example-result.txt
aws --endpoint-url http://192.168.1.9:9000 s3 rm s3://database/example.txt
aws --endpoint-url http://192.168.1.9:9000 s3api delete-bucket --bucket database