use actix_web::{error, web};

use crate::client::TomatoClient;

#[derive(Clone)]
pub struct WebState {
    client: TomatoClient,
}

impl WebState {
    pub fn new(client: TomatoClient) -> WebState {
        WebState { client }
    }
}

pub async fn metrics(data: web::Data<WebState>) -> Result<String, error::Error> {
    data.client
        .get_metrics()
        .await
        .map(|resp| resp.to_string())
        .map_err(|err| error::ErrorInternalServerError(err))
}
