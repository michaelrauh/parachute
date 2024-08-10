echo "/////////////////////////////////////////////////////////////////////////////////"
cargo run --release -- --add example.txt --location database --endpoint http://192.168.1.11:9000
cargo run --release -- --location database --endpoint http://192.168.1.11:9000
cargo run --release -- --location database --endpoint http://192.168.1.11:9000 --get
cargo run --release -- --location database --endpoint http://192.168.1.11:9000 --delete
