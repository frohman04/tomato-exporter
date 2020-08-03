use std::fs;

use serde::Deserialize;

pub fn load_conf() -> Config {
    let conf_str = fs::read_to_string("conf.yaml").expect("Unable to find config file");
    let conf: Config = serde_yaml::from_str(conf_str.as_str()).expect("Unable to load config file");
    conf
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    ip: String,
    port: u16,
    modules: Modules,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Modules {
    mod_bandwidth: Option<ModBandwidth>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct ModBandwidth {
    slug: String,
    router_ip: String,
    admin_username: String,
    admin_password: String,
    http_id: String,
}
