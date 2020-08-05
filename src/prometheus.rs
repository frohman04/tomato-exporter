use actix_web::{error, web};

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

pub async fn metrics(data: web::Data<WebState>) -> Result<String, error::Error> {
    let raw_metrics = data
        .bandwidth_client
        .get_bandwidth()
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let resp = PromResponse::new(vec![PromMetric::new(
        "bandwidth",
        "The number of bytes transmitted over an interface",
        PromMetricType::Counter,
        raw_metrics
            .iter()
            .map(|(key, value)| {
                vec![
                    PromSample::new(
                        vec![
                            PromLabel::new("if", key.to_string()),
                            PromLabel::new("direction", "rx".to_string()),
                        ],
                        value.to_owned().rx as f64,
                        None,
                    ),
                    PromSample::new(
                        vec![
                            PromLabel::new("if", key.to_string()),
                            PromLabel::new("direction", "tx".to_string()),
                        ],
                        value.to_owned().tx as f64,
                        None,
                    ),
                ]
            })
            .flatten()
            .collect(),
    )]);
    info!("{:?}", resp);

    Ok("hello world".to_string())
}

#[derive(PartialEq, PartialOrd, Debug)]
struct PromResponse {
    metrics: Vec<PromMetric>,
}

impl PromResponse {
    pub fn new(metrics: Vec<PromMetric>) -> PromResponse {
        PromResponse { metrics }
    }
}

#[derive(Eq, PartialEq, PartialOrd, Debug)]
#[allow(dead_code)]
enum PromMetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
    Untyped,
}

#[derive(PartialEq, PartialOrd, Debug)]
struct PromMetric {
    name: String,
    help: String,
    typ: PromMetricType,
    samples: Vec<PromSample>,
}

impl PromMetric {
    pub fn new(
        name: &str,
        help: &str,
        typ: PromMetricType,
        samples: Vec<PromSample>,
    ) -> PromMetric {
        PromMetric {
            name: name.to_string(),
            help: help.to_string(),
            typ,
            samples,
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
struct PromSample {
    labels: Vec<PromLabel>,
    value: f64,
    timestamp: Option<u64>,
}

impl PromSample {
    pub fn new(labels: Vec<PromLabel>, value: f64, timestamp: Option<u64>) -> PromSample {
        PromSample {
            labels,
            value,
            timestamp,
        }
    }
}

#[derive(Eq, PartialEq, PartialOrd, Debug)]
struct PromLabel {
    name: String,
    value: String,
}

impl PromLabel {
    pub fn new(name: &str, value: String) -> PromLabel {
        PromLabel {
            name: name.to_string(),
            value,
        }
    }
}
