echo "/////////////////////////////////////////////////////////////////////////////////"
cargo run --release -- --add discontinuous.txt --location database --endpoint http://192.168.1.11:9000
cargo run --release -- --location database --endpoint http://192.168.1.11:9000
cargo run --release -- --location database --endpoint http://192.168.1.11:9000 --get
cargo run --release -- --location database --endpoint http://192.168.1.11:9000 --delete
echo "/////////////////////////////////////////////////////////////////////////////////"
cargo run --release -- --add discontinuous_2.txt --location database --endpoint http://192.168.1.11:9000
cargo run --release -- --location database --endpoint http://192.168.1.11:9000
cargo run --release -- --location database --endpoint http://192.168.1.11:9000 --get
cargo run --release -- --location database --endpoint http://192.168.1.11:9000 --delete
echo "/////////////////////////////////////////////////////////////////////////////////"
cargo run --release -- --add discontinuous_3.txt --location database --endpoint http://192.168.1.11:9000
cargo run --release -- --location database --endpoint http://192.168.1.11:9000
cargo run --release -- --location database --endpoint http://192.168.1.11:9000 --get
cargo run --release -- --location database --endpoint http://192.168.1.11:9000 --delete
echo "/////////////////////////////////////////////////////////////////////////////////"
cargo run --release -- --add discontinuous_4.txt --location database --endpoint http://192.168.1.11:9000
cargo run --release -- --location database --endpoint http://192.168.1.11:9000
cargo run --release -- --location database --endpoint http://192.168.1.11:9000 --get
cargo run --release -- --location database --endpoint http://192.168.1.11:9000 --delete
echo "/////////////////////////////////////////////////////////////////////////////////"