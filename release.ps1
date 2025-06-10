pwsh -Command {
    $ErrorActionPreference='Stop'
    Set-PSDebug -Trace 1

    cargo nextest run --no-tests=warn

    $env:CROSS_CONTAINER_UID=0
    $env:CROSS_CONTAINER_GID=0
    cross build --release --target aarch64-unknown-linux-gnu

    scp conf.json target/aarch64-unknown-linux-gnu/release/tomato-exporter gilneas:~
    ssh gilneas -- "chmod +x ~/tomato-exporter && sudo mv ~/tomato-exporter /usr/local/bin/tomato_exporter && sudo mv ~/conf.json /etc/tomato-exporter/conf.json && sudo systemctl restart tomato_exporter"
}
