extern crate actix_web;
#[macro_use]
extern crate log;
extern crate regex;
extern crate reqwest;
extern crate serde_json;
extern crate serde_yaml;
extern crate simplelog;

#[cfg(test)]
#[macro_use]
extern crate maplit;

mod bandwidth;
mod config;

use actix_web::{App, HttpServer};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
    )])
    .unwrap();

    let conf = config::load_conf();
    let client = conf
        .modules
        .mod_bandwidth
        .map(|c| {
            bandwidth::BandwidthClient::new(
                c.router_ip,
                c.admin_username,
                c.admin_password,
                c.http_id,
            )
        })
        .expect("Must define mod_bandwidth configuration");

    HttpServer::new(|| App::new())
        .bind(format!("{}:{}", conf.ip, conf.port))?
        .run()
        .await
}
