#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct PromResponse {
    metrics: Vec<PromMetric>,
}

impl PromResponse {
    pub fn new(metrics: Vec<PromMetric>) -> PromResponse {
        PromResponse { metrics }
    }

    pub fn to_prom(&self) -> String {
        self.metrics
            .iter()
            .map(|metric| metric.to_prom())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[derive(Eq, PartialEq, PartialOrd, Debug, Clone)]
#[allow(dead_code)]
pub enum PromMetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
    Untyped,
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct PromMetric {
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

    pub fn to_prom(&self) -> String {
        format!(
            "# HELP {} {}\n# TYPE {} {}\n{}",
            self.name,
            self.help,
            self.name,
            format!("{:?}", self.typ).to_lowercase(),
            self.samples
                .iter()
                .map(|sample| sample.to_prom(self.name.clone()))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct PromSample {
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

    pub fn to_prom(&self, name: String) -> String {
        format!(
            "{}{{{}}} {}{}",
            name,
            self.labels
                .iter()
                .map(|label| label.to_prom())
                .collect::<Vec<String>>()
                .join(","),
            self.value.to_string(),
            self.timestamp
                .map_or_else(|| "".to_string(), |ts| format!(" {}", ts.to_string()))
        )
    }
}

#[derive(Eq, PartialEq, PartialOrd, Debug, Clone)]
pub struct PromLabel {
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

    pub fn to_prom(&self) -> String {
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
        assert_eq!(label.to_prom(), "foo=\"bar\"")
    }

    #[test]
    fn test__PromSample__to_string__no_labels_no_timestamp() {
        let sample = PromSample::new(vec![], 4.5, None);
        assert_eq!(sample.to_prom("baz".to_string()), "baz{} 4.5")
    }

    #[test]
    fn test__PromSample__to_string__no_labels_with_timestamp() {
        let sample = PromSample::new(vec![], 4.5, Some(12345));
        assert_eq!(sample.to_prom("baz".to_string()), "baz{} 4.5 12345")
    }

    #[test]
    fn test__PromSample__to_string__one_label_no_timestamp() {
        let sample = PromSample::new(vec![PromLabel::new("foo", "bar".to_string())], 4.5, None);
        assert_eq!(sample.to_prom("baz".to_string()), "baz{foo=\"bar\"} 4.5")
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
            sample.to_prom("baz".to_string()),
            "baz{foo=\"bar\",go=\"bucks\"} 4.5"
        )
    }

    #[test]
    fn test__PromMetric__to_string__no_samples() {
        let metric = PromMetric::new("baz", "A funny value", PromMetricType::Counter, vec![]);
        assert_eq!(
            metric.to_prom(),
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
            metric.to_prom(),
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
            metric.to_prom(),
            "# HELP baz A funny value\n# TYPE baz counter\nbaz{foo=\"bar\"} 4.5\nbaz{} 4.5 12345"
        )
    }

    #[test]
    fn test__PromResponse__to_string__no_metrics() {
        let response = PromResponse::new(vec![]);
        assert_eq!(response.to_prom(), "")
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
            response.to_prom(),
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
            response.to_prom(),
            "# HELP baz A funny value\n# TYPE baz counter\nbaz{foo=\"bar\"} 4.5\n# HELP spam A silly value\n# TYPE spam counter\nspam{bar=\"foo\"} 5.4"
        )
    }
}
