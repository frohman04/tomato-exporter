use std::collections::BTreeMap;

use regex::Regex;

use crate::client::{DataClient, TomatoClientInternal};
use crate::prometheus::{PromMetric, PromMetricType, PromSample};

#[derive(Clone)]
pub struct MemClient {
    client: TomatoClientInternal,
}

impl MemClient {
    pub fn new(client: TomatoClientInternal) -> MemClient {
        MemClient { client }
    }

    async fn get_mem(&self) -> Result<BTreeMap<String, u64>, reqwest::Error> {
        let body = self
            .client
            .run_command("cat /proc/meminfo".to_string())
            .await?;
        Ok(MemClient::parse_body(body))
    }

    fn parse_body(body: String) -> BTreeMap<String, u64> {
        let mem_re = Regex::new(r"(?P<name>[^:\n]+):\s+(?P<val_kB>[0-9]+) kB").unwrap();
        mem_re
            .captures_iter(body.as_str())
            .map(|capture| {
                (
                    capture.name("name").unwrap().as_str().to_string(),
                    capture
                        .name("val_kB")
                        .unwrap()
                        .as_str()
                        .parse::<u64>()
                        .unwrap()
                        * 1024,
                )
            })
            .collect()
    }

    fn raw_to_prom(raw_metrics: BTreeMap<String, u64>) -> Vec<PromMetric> {
        raw_metrics
            .into_iter()
            .map(|(name, val_bytes)| {
                PromMetric::new(
                    format!("node_memory_{}_bytes", name).as_str(),
                    format!("Memory information field {}_bytes", name).as_str(),
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), val_bytes as f64, None)],
                )
            })
            .collect()
    }
}

#[async_trait]
impl DataClient for MemClient {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, reqwest::Error> {
        let raw_metrics = self.get_mem().await?;
        Ok(MemClient::raw_to_prom(raw_metrics))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_body() {
        let body = "MemTotal:       255700 kB
MemFree:        221240 kB
Buffers:          5312 kB
Cached:          15428 kB
SwapCached:          0 kB
Active:           9976 kB
Inactive:        13284 kB
HighTotal:      131072 kB
HighFree:       109608 kB
LowTotal:       124628 kB
LowFree:        111632 kB
SwapTotal:           0 kB
SwapFree:            0 kB
Dirty:               0 kB
Writeback:           0 kB
AnonPages:        2524 kB
Mapped:           1900 kB
Slab:             6464 kB
SReclaimable:      988 kB
SUnreclaim:       5476 kB
PageTables:        296 kB
NFS_Unstable:        0 kB
Bounce:              0 kB
WritebackTmp:        0 kB
CommitLimit:    255700 kB
Committed_AS:     5908 kB
VmallocTotal:  1015800 kB
VmallocUsed:      3944 kB
VmallocChunk:  1008828 kB";
        assert_eq!(
            MemClient::parse_body(body.to_string()),
            btreemap! {
                "MemTotal".to_string() => 255700 * 1024,
                "MemFree".to_string() => 221240 * 1024,
                "Buffers".to_string() => 5312 * 1024,
                "Cached".to_string() => 15428 * 1024,
                "SwapCached".to_string() => 0,
                "Active".to_string() => 9976 * 1024,
                "Inactive".to_string() => 13284 * 1024,
                "HighTotal".to_string() => 131072 * 1024,
                "HighFree".to_string() => 109608 * 1024,
                "LowTotal".to_string() => 124628 * 1024,
                "LowFree".to_string() => 111632 * 1024,
                "SwapTotal".to_string() => 0,
                "SwapFree".to_string() => 0,
                "Dirty".to_string() => 0,
                "Writeback".to_string() => 0,
                "AnonPages".to_string() => 2524 * 1024,
                "Mapped".to_string() => 1900 * 1024,
                "Slab".to_string() => 6464 * 1024,
                "SReclaimable".to_string() => 988 * 1024,
                "SUnreclaim".to_string() => 5476 * 1024,
                "PageTables".to_string() => 296 * 1024,
                "NFS_Unstable".to_string() => 0,
                "Bounce".to_string() => 0,
                "WritebackTmp".to_string() => 0,
                "CommitLimit".to_string() => 255700 * 1024,
                "Committed_AS".to_string() => 5908 * 1024,
                "VmallocTotal".to_string() => 1015800 * 1024,
                "VmallocUsed".to_string() => 3944 * 1024,
                "VmallocChunk".to_string() => 1008828 * 1024,
            }
        )
    }

    #[test]
    fn test_raw_to_prom() {
        assert_eq!(
            MemClient::raw_to_prom(btreemap! {
                "MemTotal".to_string() => 255700 * 1024,
                "MemFree".to_string() => 221240 * 1024,
                "Buffers".to_string() => 5312 * 1024,
                "Cached".to_string() => 15428 * 1024,
            }),
            vec![
                PromMetric::new(
                    "node_memory_Buffers_bytes",
                    "Memory information field Buffers_bytes",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), (5312 * 1024) as f64, None)],
                ),
                PromMetric::new(
                    "node_memory_Cached_bytes",
                    "Memory information field Cached_bytes",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), (15428 * 1024) as f64, None)],
                ),
                PromMetric::new(
                    "node_memory_MemFree_bytes",
                    "Memory information field MemFree_bytes",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), (221240 * 1024) as f64, None)],
                ),
                PromMetric::new(
                    "node_memory_MemTotal_bytes",
                    "Memory information field MemTotal_bytes",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), (255700 * 1024) as f64, None)],
                ),
            ]
        )
    }
}
