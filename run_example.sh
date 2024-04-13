set -ex

aws --endpoint-url http://192.168.1.9:9000 s3api create-bucket --bucket database

cargo run -- --add princess.txt --location database --endpoint http://192.168.1.9:9000
aws --endpoint-url http://192.168.1.9:9000 s3 ls database/chunks/

aws --endpoint-url http://192.168.1.9:9000 s3 rm --recursive s3://database/chunks/
aws --endpoint-url http://192.168.1.9:9000 s3api delete-bucket --bucket database