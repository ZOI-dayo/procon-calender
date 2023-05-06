cd `dirname $0`
cargo build --release
# screen -d -S procon-calender -m ./target/release/procon-calender
./target/release/procon-calender
