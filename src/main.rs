#![forbid(unsafe_code)]

extern crate actix_web;
extern crate ansi_term;
#[macro_use]
extern crate async_trait;
extern crate clap;
extern crate dyn_clone;
extern crate futures;
#[macro_use]
extern crate maplit;
extern crate regex;
extern crate reqwest;
extern crate serde_yaml;
extern crate tracing;
extern crate tracing_actix_web;
extern crate tracing_log;
extern crate tracing_subscriber;
extern crate url;

mod client;
mod config;
mod prometheus;
mod web;

use actix_web::middleware::{Compress, Logger};
use actix_web::web::Data;
use actix_web::{web as a_web, App, HttpServer};
use clap::{crate_name, crate_version};
use tracing::{info, Level};
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

use web::{metrics, WebState};

use client::TomatoClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ansi_enabled = fix_ansi_term();
    LogTracer::init().expect("routing log to tracing failed");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_ansi(ansi_enabled)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

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

    let conf = config::load_conf(matches.get_one::<String>("conf").unwrap().clone());
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
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(Compress::default())
            .app_data(Data::new(WebState::new(client.clone())))
            .route(path.as_str(), a_web::get().to(metrics))
    })
    .bind(format!("{}:{}", conf.ip, conf.port))?
    .run()
    .await
}

#[cfg(target_os = "windows")]
fn fix_ansi_term() -> bool {
    ansi_term::enable_ansi_support().map_or(false, |()| true)
}

#[cfg(not(target_os = "windows"))]
fn fix_ansi_term() -> bool {
    true
}
