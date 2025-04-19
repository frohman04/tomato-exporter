#!/bin/sh

set -e
set -x

cargo nextest run
if [ -z ${OS+x} ]; then
    # in Linux, use cross to build other platforms
    CROSS_CONTAINER_UID=0 CROSS_CONTAINER_GID=0 cross build --release --target aarch64-unknown-linux-gnu
else
    # in Windows, so need to uze zigbuild for aarch64 compat
    cargo zigbuild --release --target aarch64-unknown-linux-gnu
fi
scp conf.json target/aarch64-unknown-linux-gnu/release/tomato-exporter gilneas:~
ssh gilneas -- "chmod +x ~/tomato-exporter && sudo mv ~/tomato-exporter /usr/local/bin/tomato_exporter && sudo mv ~/conf.json /etc/tomato-exporter/conf.json && sudo systemctl restart tomato_exporter"
