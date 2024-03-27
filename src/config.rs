use std::fs;

use serde::Deserialize;

pub fn load_conf(path: String) -> Config {
    let conf_str = fs::read_to_string(path).expect("Unable to find config file");
    let conf: Config = serde_json::from_str(conf_str.as_str()).expect("Unable to load config file");
    conf
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
pub struct Config {
    pub ip: String,
    pub port: u16,
    pub slug: String,
    pub router_ip: String,
    pub admin_username: String,
    pub admin_password: String,
    pub http_id: String,
}
