use regex::{Captures, Regex};

use crate::client::{Scraper, TomatoClientInternal};
use crate::prometheus::{PromMetric, PromMetricType, PromSample};

#[derive(Clone)]
pub struct LoadClient {
    client: TomatoClientInternal,
}

#[derive(Debug, PartialEq)]
struct LoadInfo {
    pub load_1m: f32,
    pub load_5m: f32,
    pub load_15m: f32,
    pub total_procs: u32,
}

impl LoadClient {
    pub fn new(client: TomatoClientInternal) -> LoadClient {
        LoadClient { client }
    }

    async fn get_time(&self) -> Result<LoadInfo, reqwest::Error> {
        let body = self
            .client
            .run_command("cat /proc/loadavg".to_string())
            .await?;
        Ok(LoadClient::parse_body(body))
    }

    fn parse_cap_f32(capture: &Captures, field: &str) -> f32 {
        capture
            .name(field)
            .unwrap()
            .as_str()
            .parse::<f32>()
            .unwrap()
    }

    fn parse_cap_u32(capture: &Captures, field: &str) -> u32 {
        capture
            .name(field)
            .unwrap()
            .as_str()
            .parse::<u32>()
            .unwrap()
    }

    fn parse_body(body: String) -> LoadInfo {
        let body_parser_re =
            Regex::new(r"(?P<load_1m>[0-9]+.[0-9]+) (?P<load_5m>[0-9]+.[0-9]+) (?P<load_15m>[0-9]+.[0-9]+) (?P<running>[0-9]+)/(?P<total_procs>[0-9]+) (?P<last_pid>[0-9]+)")
                .unwrap();
        body_parser_re
            .captures(body.as_str().trim())
            .map(|capture| LoadInfo {
                load_1m: LoadClient::parse_cap_f32(&capture, "load_1m"),
                load_5m: LoadClient::parse_cap_f32(&capture, "load_5m"),
                load_15m: LoadClient::parse_cap_f32(&capture, "load_15m"),
                total_procs: LoadClient::parse_cap_u32(&capture, "total_procs"),
            })
            .expect("Unable to parse load")
    }

    fn raw_to_prom(raw_metrics: LoadInfo) -> Vec<PromMetric> {
        vec![
            PromMetric::new(
                "node_load1",
                "1m load average",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    Vec::new(),
                    raw_metrics.load_1m as f64,
                    None,
                )],
            ),
            PromMetric::new(
                "node_load5",
                "5m load average",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    Vec::new(),
                    raw_metrics.load_5m as f64,
                    None,
                )],
            ),
            PromMetric::new(
                "node_load15",
                "15m load average",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    Vec::new(),
                    raw_metrics.load_15m as f64,
                    None,
                )],
            ),
            PromMetric::new(
                "node_processes_pids",
                "Number of PIDs",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    Vec::new(),
                    raw_metrics.total_procs as f64,
                    None,
                )],
            ),
        ]
    }
}

#[async_trait]
impl Scraper for LoadClient {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, reqwest::Error> {
        let raw_metrics = self.get_time().await?;
        Ok(LoadClient::raw_to_prom(raw_metrics))
    }

    fn get_name(&self) -> String {
        "load".to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_body() {
        let body = "0.01 0.02 0.03 2/38 23618";
        assert_eq!(
            LoadClient::parse_body(body.to_string()),
            LoadInfo {
                load_1m: 0.01f32,
                load_5m: 0.02f32,
                load_15m: 0.03f32,
                total_procs: 38u32,
            }
        )
    }

    #[test]
    fn test_raw_to_prom() {
        assert_eq!(
            LoadClient::raw_to_prom(LoadInfo {
                load_1m: 0.01f32,
                load_5m: 0.02f32,
                load_15m: 0.03f32,
                total_procs: 38u32,
            }),
            vec![
                PromMetric::new(
                    "node_load1",
                    "1m load average",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), 0.01f32 as f64, None,)],
                ),
                PromMetric::new(
                    "node_load5",
                    "5m load average",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), 0.02f32 as f64, None,)],
                ),
                PromMetric::new(
                    "node_load15",
                    "15m load average",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), 0.03f32 as f64, None,)],
                ),
                PromMetric::new(
                    "node_processes_pids",
                    "Number of PIDs",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), 38 as f64, None,)],
                ),
            ]
        )
    }
}
