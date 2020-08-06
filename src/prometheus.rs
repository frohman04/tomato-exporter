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

    Ok(resp.to_string())
}

#[derive(PartialEq, PartialOrd, Debug)]
struct PromResponse {
    metrics: Vec<PromMetric>,
}

impl PromResponse {
    pub fn new(metrics: Vec<PromMetric>) -> PromResponse {
        PromResponse { metrics }
    }

    pub fn to_string(&self) -> String {
        self.metrics
            .iter()
            .map(|metric| metric.to_string())
            .collect::<Vec<String>>()
            .join("\n")
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

    pub fn to_string(&self) -> String {
        format!(
            "# HELP {} {}\n# TYPE {} {}\n{}",
            self.name,
            self.help,
            self.name,
            format!("{:?}", self.typ).to_lowercase(),
            self.samples
                .iter()
                .map(|sample| sample.to_string(self.name.clone()))
                .collect::<Vec<String>>()
                .join("\n")
        )
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

    pub fn to_string(&self, name: String) -> String {
        format!(
            "{}{{{}}} {}{}",
            name,
            self.labels
                .iter()
                .map(|label| label.to_string())
                .collect::<Vec<String>>()
                .join(","),
            self.value.to_string(),
            self.timestamp
                .map_or_else(|| "".to_string(), |ts| format!(" {}", ts.to_string()))
        )
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

    pub fn to_string(&self) -> String {
        format!("{}=\"{}\"", self.name, self.value)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use super::*;

    #[test]
    fn test__PromLabel__to_string() {
        let label = PromLabel::new("foo", "bar".to_string());
        assert_eq!(label.to_string(), "foo=\"bar\"")
    }

    #[test]
    fn test__PromSample__to_string__no_labels_no_timestamp() {
        let sample = PromSample::new(vec![], 4.5, None);
        assert_eq!(sample.to_string("baz".to_string()), "baz{} 4.5")
    }

    #[test]
    fn test__PromSample__to_string__no_labels_with_timestamp() {
        let sample = PromSample::new(vec![], 4.5, Some(12345));
        assert_eq!(sample.to_string("baz".to_string()), "baz{} 4.5 12345")
    }

    #[test]
    fn test__PromSample__to_string__one_label_no_timestamp() {
        let sample = PromSample::new(vec![PromLabel::new("foo", "bar".to_string())], 4.5, None);
        assert_eq!(sample.to_string("baz".to_string()), "baz{foo=\"bar\"} 4.5")
    }

    #[test]
    fn test__PromSample__to_string__many_labels_no_timestamp() {
        let sample = PromSample::new(
            vec![
                PromLabel::new("foo", "bar".to_string()),
                PromLabel::new("go", "bucks".to_string()),
            ],
            4.5,
            None,
        );
        assert_eq!(
            sample.to_string("baz".to_string()),
            "baz{foo=\"bar\",go=\"bucks\"} 4.5"
        )
    }

    #[test]
    fn test__PromMetric__to_string__no_samples() {
        let metric = PromMetric::new("baz", "A funny value", PromMetricType::Counter, vec![]);
        assert_eq!(
            metric.to_string(),
            "# HELP baz A funny value\n# TYPE baz counter\n"
        )
    }

    #[test]
    fn test__PromMetric__to_string__one_sample() {
        let metric = PromMetric::new(
            "baz",
            "A funny value",
            PromMetricType::Counter,
            vec![PromSample::new(
                vec![PromLabel::new("foo", "bar".to_string())],
                4.5,
                None,
            )],
        );
        assert_eq!(
            metric.to_string(),
            "# HELP baz A funny value\n# TYPE baz counter\nbaz{foo=\"bar\"} 4.5"
        )
    }

    #[test]
    fn test__PromMetric__to_string__many_samples() {
        let metric = PromMetric::new(
            "baz",
            "A funny value",
            PromMetricType::Counter,
            vec![
                PromSample::new(vec![PromLabel::new("foo", "bar".to_string())], 4.5, None),
                PromSample::new(vec![], 4.5, Some(12345)),
            ],
        );
        assert_eq!(
            metric.to_string(),
            "# HELP baz A funny value\n# TYPE baz counter\nbaz{foo=\"bar\"} 4.5\nbaz{} 4.5 12345"
        )
    }

    #[test]
    fn test__PromResponse__to_string__no_metrics() {
        let response = PromResponse::new(vec![]);
        assert_eq!(response.to_string(), "")
    }

    #[test]
    fn test__PromResponse__to_string__one_metric() {
        let response = PromResponse::new(vec![PromMetric::new(
            "baz",
            "A funny value",
            PromMetricType::Counter,
            vec![PromSample::new(
                vec![PromLabel::new("foo", "bar".to_string())],
                4.5,
                None,
            )],
        )]);
        assert_eq!(
            response.to_string(),
            "# HELP baz A funny value\n# TYPE baz counter\nbaz{foo=\"bar\"} 4.5"
        )
    }

    #[test]
    fn test__PromResponse__to_string__many_metrics() {
        let response = PromResponse::new(vec![
            PromMetric::new(
                "baz",
                "A funny value",
                PromMetricType::Counter,
                vec![PromSample::new(
                    vec![PromLabel::new("foo", "bar".to_string())],
                    4.5,
                    None,
                )],
            ),
            PromMetric::new(
                "spam",
                "A silly value",
                PromMetricType::Counter,
                vec![PromSample::new(
                    vec![PromLabel::new("bar", "foo".to_string())],
                    5.4,
                    None,
                )],
            ),
        ]);
        assert_eq!(
            response.to_string(),
            "# HELP baz A funny value\n# TYPE baz counter\nbaz{foo=\"bar\"} 4.5\n# HELP spam A silly value\n# TYPE spam counter\nspam{bar=\"foo\"} 5.4"
        )
    }
}
