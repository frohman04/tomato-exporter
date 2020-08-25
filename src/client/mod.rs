mod bandwidth;
mod cpu;
mod node;
mod time;
mod uname;

use std::collections::HashMap;

use dyn_clone::DynClone;
use futures::future::join_all;
use reqwest::{Client, ClientBuilder};
use url::form_urlencoded;

use crate::client::bandwidth::BandwidthClient;
use crate::client::cpu::CpuClient;
use crate::client::node::NodeClient;
use crate::client::time::TimeClient;
use crate::client::uname::UnameClient;
use crate::prometheus::{PromMetric, PromResponse};

#[async_trait]
trait DataClient: DynClone + Send {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, reqwest::Error>;
}

dyn_clone::clone_trait_object!(DataClient);

#[derive(Clone)]
pub struct TomatoClient {
    data_clients: Vec<Box<dyn DataClient>>,
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
                Box::new(BandwidthClient::new(client.clone())),
                Box::new(CpuClient::new(client.clone())),
                Box::new(NodeClient::new(client.clone())),
                Box::new(TimeClient::new(client.clone())),
                Box::new(UnameClient::new(client.clone())),
            ],
        }
    }

    pub async fn get_metrics(&self) -> Result<PromResponse, reqwest::Error> {
        let results = join_all(self.data_clients.iter().map(|client| client.get_metrics()))
            .await
            .into_iter()
            .collect::<Result<Vec<Vec<PromMetric>>, reqwest::Error>>()?;
        let metrics = results.into_iter().flatten().collect();

        Ok(PromResponse::new(metrics))
    }
}

#[derive(Clone)]
pub struct TomatoClientInternal {
    hostname: String,
    admin_username: String,
    admin_password: String,
    http_id: String,
    client: Client,
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
            client: ClientBuilder::new()
                .build()
                .expect("Unable to construct HTTP client"),
        }
    }

    pub async fn make_request(
        &self,
        endpoint: String,
        args: Option<HashMap<String, String>>,
    ) -> Result<String, reqwest::Error> {
        let arg_map = args.unwrap_or_else(|| HashMap::new());
        let body = arg_map
            .iter()
            .fold(
                form_urlencoded::Serializer::new(String::new())
                    .append_pair("_http_id", self.http_id.as_str()),
                |bb, (key, value)| bb.append_pair(key.as_str().clone(), value.as_str().clone()),
            )
            .finish();

        let response = self
            .client
            .post(format!("{}/{}", &self.hostname.clone(), endpoint).as_str())
            .basic_auth(
                self.admin_username.clone(),
                Some(self.admin_password.clone()),
            )
            .body(body)
            .send()
            .await?;
        Ok(response.text().await?)
    }

    async fn run_command(&self, command: String) -> Result<String, reqwest::Error> {
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
