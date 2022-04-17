#![forbid(unsafe_code)]

extern crate actix_web;
#[macro_use]
extern crate async_trait;
extern crate clap;
extern crate dyn_clone;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;
extern crate regex;
extern crate reqwest;
extern crate serde_yaml;
extern crate url;

mod client;
mod config;
mod prometheus;
mod web;

use actix_web::middleware::{Compress, Logger};
use actix_web::{web as a_web, App, HttpServer};
use actix_web::web::Data;
use clap::{crate_name, crate_version};
use env_logger::Env;

use web::{metrics, WebState};

use client::TomatoClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let matches = clap::Command::new("tomato_exporter")
        .version(crate_version!())
        .author("Chris Lieb")
        .arg(
            clap::Arg::new("conf")
                .short('c')
                .long("conf")
                .default_value("conf.yaml"),
        )
        .get_matches();

    let conf = config::load_conf(matches.value_of("conf").unwrap().to_string());
    info!(
        "Starting {} v{}: http://{}:{}/{}",
        crate_name!(),
        crate_version!(),
        conf.ip,
        conf.port,
        conf.slug
    );

    let client = TomatoClient::new(
        conf.router_ip,
        conf.admin_username,
        conf.admin_password,
        conf.http_id,
    );

    let path = format!("/{}", conf.slug.clone());
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .app_data(Data::new(WebState::new(client.clone())))
            .route(path.as_str(), a_web::get().to(metrics))
    })
    .bind(format!("{}:{}", conf.ip, conf.port))?
    .run()
    .await
}
