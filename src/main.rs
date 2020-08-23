extern crate actix_web;
#[macro_use]
extern crate async_trait;
extern crate clap;
extern crate dyn_clone;
extern crate futures;
#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;
extern crate regex;
extern crate reqwest;
extern crate serde_json;
extern crate serde_yaml;
extern crate simplelog;
extern crate url;

mod config;
mod modules;
mod prometheus;
mod web;

use actix_web::middleware::{Compress, Logger};
use actix_web::{web as a_web, App, HttpServer};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};

use web::{metrics, WebState};

use modules::bandwidth::BandwidthClient;
use modules::node::NodeClient;
use modules::tomato::TomatoClient;

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
        .map(|c| TomatoClient::new(c.router_ip, c.admin_username, c.admin_password, c.http_id))
        .expect("Must define mod_bandwidth configuration");
    let bandwidth_client = BandwidthClient::new(client.clone());
    let node_client = NodeClient::new(client.clone());

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .data(WebState::new(vec![
                Box::new(bandwidth_client.clone()),
                Box::new(node_client.clone()),
            ]))
            .route("/metrics", a_web::get().to(metrics))
    })
    .bind(format!("{}:{}", conf.ip, conf.port))?
    .run()
    .await
}
