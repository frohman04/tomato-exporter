use std::collections::HashMap;

use reqwest::{Client, ClientBuilder};
use url::form_urlencoded;

#[derive(Clone)]
pub struct TomatoClient {
    hostname: String,
    admin_username: String,
    admin_password: String,
    http_id: String,
    client: Client,
}

impl TomatoClient {
    pub fn new(
        ip_address: String,
        admin_username: String,
        admin_password: String,
        http_id: String,
    ) -> TomatoClient {
        info!("Creating TomatoUSB client for {}", ip_address);
        TomatoClient {
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
}
