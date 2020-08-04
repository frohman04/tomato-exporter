use actix_web::web;

use crate::bandwidth::BandwidthClient;

#[derive(Clone)]
pub struct WebState {
    bandwidth_client: BandwidthClient,
}

impl WebState {
    pub fn new(bandwidth_client: BandwidthClient) -> WebState {
        WebState { bandwidth_client }
    }
}

pub async fn metrics(data: web::Data<WebState>) -> String {
    "hello world".to_string()
}
