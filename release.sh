#!/bin/sh

set -e

export ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_LIB_DIR=/c/SysGCC/raspberry/arm-linux-gnueabihf/sysroot/usr/lib/arm-linux-gnueabihf
export ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_INCLUDE_DIR=/c/SysGCC/raspberry/arm-linux-gnueabihf/sysroot/usr/include/openssl
export PATH=/c/SysGCC/raspberry/bin:/c/SysGCC/raspberry/arm-linux-gnueabihf/bin:${PATH}

set -x

cargo test
cargo build --release --target armv7-unknown-linux-gnueabihf
scp conf.yaml target/armv7-unknown-linux-gnueabihf/release/tomato-exporter gilneas:~
ssh gilneas -- "chmod +x ~/tomato-exporter && sudo mv ~/tomato-exporter /usr/local/bin/tomato_exporter && sudo mv ~/conf.yaml /etc/tomato-exporter/conf.yaml && sudo systemctl restart tomato_exporter"
