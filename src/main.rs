extern crate actix_web;
extern crate clap;
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
mod prometheus;

use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, HttpServer};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};

use prometheus::{metrics, WebState};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
    )])
    .unwrap();

    let matches = clap::App::new("tomato_exporter")
        .version("0.1")
        .author("Chris Lieb")
        .arg(
            clap::Arg::with_name("conf")
                .short("c")
                .long("conf")
                .default_value("conf.yaml"),
        )
        .get_matches();

    let conf = config::load_conf(matches.value_of("conf").unwrap().to_string());
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

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .data(WebState::new(client.clone()))
            .route("/metrics", web::get().to(metrics))
    })
    .bind(format!("{}:{}", conf.ip, conf.port))?
    .run()
    .await
}
