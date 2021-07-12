use regex::Regex;

use crate::client::{Scraper, TomatoClientInternal};
use crate::prometheus::{PromMetric, PromMetricType, PromSample};

#[derive(Clone)]
pub struct TimeClient {
    client: TomatoClientInternal,
}

#[derive(Debug, PartialEq)]
struct Times {
    pub curr_timestamp: u64,
    pub up_timestamp: u64,
}

impl TimeClient {
    pub fn new(client: TomatoClientInternal) -> TimeClient {
        TimeClient { client }
    }

    async fn get_time(&self) -> Result<Times, reqwest::Error> {
        let body = self
            .client
            .run_command("date +%s && cat /proc/uptime".to_string())
            .await?;
        Ok(TimeClient::parse_body(body))
    }

    fn parse_body(body: String) -> Times {
        let body_parser_re =
            Regex::new(r"(?s)(?P<timestamp>[0-9]+)\n(?P<up_seconds>[0-9]+\.[0-9]+) [0-9]+\.[0-9]+")
                .unwrap();
        body_parser_re
            .captures(body.as_str().trim())
            .map(|capture| {
                let curr_timestamp = capture
                    .name("timestamp")
                    .unwrap()
                    .as_str()
                    .parse::<u64>()
                    .unwrap();
                let up_seconds = capture
                    .name("up_seconds")
                    .unwrap()
                    .as_str()
                    .parse::<f64>()
                    .unwrap() as u64;
                Times {
                    curr_timestamp,
                    up_timestamp: curr_timestamp - up_seconds,
                }
            })
            .expect("Unable to parse times")
    }

    fn raw_to_prom(raw_metrics: Times) -> Vec<PromMetric> {
        vec![
            PromMetric::new(
                "node_time_seconds",
                "System time in seconds since epoch (1970)",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    Vec::new(),
                    raw_metrics.curr_timestamp as f64,
                    None,
                )],
            ),
            PromMetric::new(
                "node_boot_time_seconds",
                "Node boot time, in unixtime",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    Vec::new(),
                    raw_metrics.up_timestamp as f64,
                    None,
                )],
            ),
        ]
    }
}

#[async_trait]
impl Scraper for TimeClient {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, reqwest::Error> {
        let raw_metrics = self.get_time().await?;
        Ok(TimeClient::raw_to_prom(raw_metrics))
    }

    fn get_name(&self) -> String {
        "time".to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_body() {
        let body = "1598394934
1810779.30 1804583.20";
        assert_eq!(
            TimeClient::parse_body(body.to_string()),
            Times {
                curr_timestamp: 1598394934u64,
                up_timestamp: 1598394934u64 - 1810779u64,
            }
        )
    }

    #[test]
    fn test_raw_to_prom() {
        assert_eq!(
            TimeClient::raw_to_prom(Times {
                curr_timestamp: 1598394934u64,
                up_timestamp: 1598394934u64 - 1810779u64,
            }),
            vec![
                PromMetric::new(
                    "node_time_seconds",
                    "System time in seconds since epoch (1970)",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), 1598394934f64, None)],
                ),
                PromMetric::new(
                    "node_boot_time_seconds",
                    "Node boot time, in unixtime",
                    PromMetricType::Gauge,
                    vec![PromSample::new(
                        Vec::new(),
                        (1598394934u64 - 1810779u64) as f64,
                        None
                    )],
                ),
            ]
        )
    }
}
