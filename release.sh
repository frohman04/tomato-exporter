#!/bin/sh

set -e
set -x

cargo test
cargo zigbuild --release --target aarch64-unknown-linux-gnu
scp conf.json target/aarch64-unknown-linux-gnu/release/tomato-exporter gilneas:~
ssh gilneas -- "chmod +x ~/tomato-exporter && sudo mv ~/tomato-exporter /usr/local/bin/tomato_exporter && sudo mv ~/conf.json /etc/tomato-exporter/conf.json && sudo systemctl restart tomato_exporter"
