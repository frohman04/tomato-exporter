use std::fs;

use serde::Deserialize;

pub fn load_conf() -> Config {
    let conf_str = fs::read_to_string("conf.yaml").expect("Unable to find config file");
    let conf: Config = serde_yaml::from_str(conf_str.as_str()).expect("Unable to load config file");
    conf
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub ip: String,
    pub port: u16,
    pub modules: Modules,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Modules {
    pub mod_bandwidth: Option<ModBandwidth>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct ModBandwidth {
    pub slug: String,
    pub router_ip: String,
    pub admin_username: String,
    pub admin_password: String,
    pub http_id: String,
}
