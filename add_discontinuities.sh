echo "/////////////////////////////////////////////////////////////////////////////////"
cargo run -- --add discontinuous.txt --location database --endpoint http://192.168.1.11:9000
cargo run -- --location database --endpoint http://192.168.1.11:9000
cargo run -- --location database --endpoint http://192.168.1.11:9000 --get
cargo run -- --location database --endpoint http://192.168.1.11:9000 --delete
