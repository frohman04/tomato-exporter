mod cpu;
mod load;
mod mem;
mod network;
mod time;
mod uname;

use std::collections::HashMap;

use ::time::OffsetDateTime;
use actix_web::client::Client;
use dyn_clone::DynClone;
use futures::future::join_all;
use url::form_urlencoded;

use crate::client::cpu::CpuClient;
use crate::client::load::LoadClient;
use crate::client::mem::MemClient;
use crate::client::network::NetworkClient;
use crate::client::time::TimeClient;
use crate::client::uname::UnameClient;
use crate::prometheus::{PromLabel, PromMetric, PromMetricType, PromResponse, PromSample};

#[async_trait]
trait Scraper: DynClone + Send {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, Box<dyn std::error::Error>>;

    fn get_name(&self) -> String;
}

dyn_clone::clone_trait_object!(Scraper);

struct ScraperResult {
    pub name: String,
    pub duration: f64,
    pub result: Result<Vec<PromMetric>, Box<dyn std::error::Error>>,
}

#[derive(Clone)]
pub struct TomatoClient {
    data_clients: Vec<Box<dyn Scraper>>,
}

impl TomatoClient {
    pub fn new(
        ip_address: String,
        admin_username: String,
        admin_password: String,
        http_id: String,
    ) -> TomatoClient {
        let client = TomatoClientInternal::new(ip_address, admin_username, admin_password, http_id);
        TomatoClient {
            data_clients: vec![
                Box::new(CpuClient::new(client.clone())),
                Box::new(LoadClient::new(client.clone())),
                Box::new(MemClient::new(client.clone())),
                Box::new(NetworkClient::new(client.clone())),
                Box::new(TimeClient::new(client.clone())),
                Box::new(UnameClient::new(client)),
            ],
        }
    }

    pub async fn get_metrics(&self) -> Result<PromResponse, Box<dyn std::error::Error>> {
        let results = join_all(
            self.data_clients
                .iter()
                .map(|scraper| TomatoClient::run_scraper(scraper.as_ref())),
        )
        .await
        .into_iter();

        let mut scraper_durations: Vec<PromSample> = Vec::new();
        let mut scraper_successes: Vec<PromSample> = Vec::new();
        let mut metrics: Vec<PromMetric> = results
            .filter_map(|result| {
                scraper_durations.push(PromSample::new(
                    vec![PromLabel::new("collector", result.name.clone())],
                    result.duration,
                    None,
                ));
                scraper_successes.push(PromSample::new(
                    vec![PromLabel::new("collector", result.name.clone())],
                    if result.result.is_ok() { 1f64 } else { 0f64 },
                    None,
                ));

                let name = result.name.clone();
                result
                    .result
                    .map_err(|err| {
                        warn!("Scraper {} failed: {}", name, err);
                        err
                    })
                    .ok()
            })
            .flatten()
            .collect();
        metrics.push(PromMetric::new(
            "node_scrape_collector_duration_seconds",
            "node_exporter: Duration of a collector scrape",
            PromMetricType::Gauge,
            scraper_durations,
        ));
        metrics.push(PromMetric::new(
            "node_scrape_collector_success",
            "Whether a collector succeeded",
            PromMetricType::Gauge,
            scraper_successes,
        ));

        Ok(PromResponse::new(metrics))
    }

    async fn run_scraper(scraper: &dyn Scraper) -> ScraperResult {
        let start_time = OffsetDateTime::now_utc();
        let result = scraper.get_metrics().await;
        let end_time = OffsetDateTime::now_utc();
        let duration = (end_time - start_time).as_seconds_f64();
        ScraperResult {
            name: scraper.get_name(),
            duration,
            result,
        }
    }
}

#[derive(Clone)]
pub struct TomatoClientInternal {
    hostname: String,
    admin_username: String,
    admin_password: String,
    http_id: String,
}

impl TomatoClientInternal {
    pub fn new(
        ip_address: String,
        admin_username: String,
        admin_password: String,
        http_id: String,
    ) -> TomatoClientInternal {
        info!("Creating TomatoUSB client for {}", ip_address);
        TomatoClientInternal {
            hostname: format!("http://{}", ip_address),
            admin_username,
            admin_password,
            http_id,
        }
    }

    pub async fn make_request(
        &self,
        endpoint: String,
        args: Option<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let arg_map = args.unwrap_or_else(HashMap::new);
        let body = arg_map
            .into_iter()
            .fold(
                form_urlencoded::Serializer::new(String::new())
                    .append_pair("_http_id", self.http_id.as_str()),
                |bb, (key, value)| bb.append_pair(key.as_str(), value.as_str()),
            )
            .finish();

        let body = {
            let client = Client::default();
            let mut response = client
                .post(format!("{}/{}", &self.hostname.clone(), endpoint).as_str())
                .basic_auth(
                    self.admin_username.clone(),
                    Some(self.admin_password.clone().as_str()),
                )
                .send_body(body)
                .await?;
            response.body().await?
        };
        Ok(std::str::from_utf8(body.as_ref()).unwrap().to_string())
    }

    async fn run_command(&self, command: String) -> Result<String, Box<dyn std::error::Error>> {
        self.make_request(
            "shell.cgi".to_string(),
            Some(hashmap! {
                "action".to_string() => "execute".to_string(),
                "nojs".to_string() => "1".to_string(),
                "working_dir".to_string() => "/www".to_string(),
                "command".to_string() => command,
            }),
        )
        .await
    }
}
