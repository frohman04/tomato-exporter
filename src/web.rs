use actix_web::{error, web};
use futures::future::join_all;

use crate::prometheus::{DataClient, PromMetric, PromResponse};

#[derive(Clone)]
pub struct WebState {
    clients: Vec<Box<dyn DataClient>>,
}

impl WebState {
    pub fn new(clients: Vec<Box<dyn DataClient>>) -> WebState {
        WebState { clients }
    }
}

pub async fn metrics(data: web::Data<WebState>) -> Result<String, error::Error> {
    let results = join_all(data.clients.iter().map(|client| client.get_metrics()))
        .await
        .into_iter()
        .collect::<Result<Vec<Vec<PromMetric>>, reqwest::Error>>()
        .map_err(|err| error::ErrorInternalServerError(err))?;
    let metrics = results.into_iter().flatten().collect();

    Ok(PromResponse::new(metrics).to_string())
}
