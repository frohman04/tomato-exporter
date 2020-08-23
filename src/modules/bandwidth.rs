use std::collections::BTreeMap;

use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer};

use crate::modules::tomato::TomatoClient;
use crate::prometheus::{DataClient, PromLabel, PromMetric, PromMetricType, PromSample};

#[derive(Clone)]
pub struct BandwidthClient {
    client: TomatoClient,
}

#[derive(Debug, Deserialize, PartialEq)]
struct BandwidthMeasurement {
    #[serde(deserialize_with = "from_hex")]
    pub rx: u64,
    #[serde(deserialize_with = "from_hex")]
    pub tx: u64,
}

fn from_hex<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    // do better hex decoding than this
    u64::from_str_radix(&s[2..], 16).map_err(D::Error::custom)
}

impl BandwidthClient {
    pub fn new(client: TomatoClient) -> BandwidthClient {
        BandwidthClient { client }
    }

    async fn get_bandwidth(
        &self,
    ) -> Result<BTreeMap<String, BandwidthMeasurement>, reqwest::Error> {
        let body = self
            .client
            .make_request(
                "update.cgi".to_string(),
                Some(hashmap! {"exec".to_string() => "netdev".to_string()}),
            )
            .await?;
        Ok(BandwidthClient::parse_body(body))
    }

    fn parse_body(body: String) -> BTreeMap<String, BandwidthMeasurement> {
        let regex = Regex::new(r"(0x[0-9a-fA-F]+)").unwrap();
        let cleaned = body
            .replace("netdev=", "")
            .replace(";", "")
            .replace("'", "\"")
            .replace("rx", "\"rx\"")
            .replace("tx", "\"tx\"");
        let cleaned = &*regex.replace_all(cleaned.as_str(), "\"$1\"");
        let parsed: BTreeMap<String, BandwidthMeasurement> =
            serde_json::from_str(cleaned).expect("Unable to parse response");
        parsed
    }

    fn raw_to_prom(raw_metrics: BTreeMap<String, BandwidthMeasurement>) -> Vec<PromMetric> {
        vec![
            PromMetric::new(
                "node_network_receive_bytes_total",
                "Network device statistic receive_bytes",
                PromMetricType::Counter,
                raw_metrics
                    .iter()
                    .map(|(key, value)| {
                        vec![PromSample::new(
                            vec![PromLabel::new("device", key.to_string())],
                            value.to_owned().rx as f64,
                            None,
                        )]
                    })
                    .flatten()
                    .collect(),
            ),
            PromMetric::new(
                "node_network_transmit_bytes_total",
                "Network device statistic transmit_bytes",
                PromMetricType::Counter,
                raw_metrics
                    .iter()
                    .map(|(key, value)| {
                        vec![PromSample::new(
                            vec![PromLabel::new("device", key.to_string())],
                            value.to_owned().tx as f64,
                            None,
                        )]
                    })
                    .flatten()
                    .collect(),
            ),
        ]
    }
}

#[async_trait]
impl DataClient for BandwidthClient {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, reqwest::Error> {
        let raw_metrics = self.get_bandwidth().await?;
        Ok(BandwidthClient::raw_to_prom(raw_metrics))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_body() {
        let body = "netdev={ \
        'eth0':{rx:0xab7666a1,tx:0x6a2c1014},\
        'vlan1':{rx:0x4c4d97a5,tx:0x839c8539},\
        'vlan2':{rx:0x2339061e,tx:0xe693c2e1},\
        'eth1':{rx:0x41122421,tx:0xd273ff5},\
        'eth2':{rx:0x5ed3a58a,tx:0xe03baf1e},\
        'br0':{rx:0xd6dd237d,tx:0x4265a458}\
        };";
        assert_eq!(
            BandwidthClient::parse_body(body.to_string()),
            btreemap! {
                "eth0".to_string() => BandwidthMeasurement { rx: 2876663457, tx: 1781272596 },
                "eth1".to_string() => BandwidthMeasurement { rx: 1091707937, tx: 220676085 },
                "eth2".to_string() => BandwidthMeasurement { rx: 1590928778, tx: 3762007838 },
                "vlan1".to_string() => BandwidthMeasurement { rx: 1280153509, tx: 2208073017 },
                "vlan2".to_string() => BandwidthMeasurement { rx: 590939678, tx: 3868443361 },
                "br0".to_string() => BandwidthMeasurement { rx: 3604816765, tx: 1113957464 },
            }
        )
    }

    #[test]
    fn test_raw_to_prom() {
        assert_eq!(
            BandwidthClient::raw_to_prom(btreemap! {
                "eth0".to_string() => BandwidthMeasurement { rx: 2876663457, tx: 1781272596 },
                "eth1".to_string() => BandwidthMeasurement { rx: 1091707937, tx: 220676085 },
                "eth2".to_string() => BandwidthMeasurement { rx: 1590928778, tx: 3762007838 },
                "vlan1".to_string() => BandwidthMeasurement { rx: 1280153509, tx: 2208073017 },
                "vlan2".to_string() => BandwidthMeasurement { rx: 590939678, tx: 3868443361 },
                "br0".to_string() => BandwidthMeasurement { rx: 3604816765, tx: 1113957464 },
            }),
            vec![
                PromMetric::new(
                    "node_network_receive_bytes_total",
                    "Network device statistic receive_bytes",
                    PromMetricType::Counter,
                    vec![
                        PromSample::new(
                            vec![PromLabel::new("device", "br0".to_string())],
                            3604816765f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth0".to_string())],
                            2876663457f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth1".to_string())],
                            1091707937f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth2".to_string())],
                            1590928778f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "vlan1".to_string())],
                            1280153509f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "vlan2".to_string())],
                            590939678f64,
                            None
                        ),
                    ],
                ),
                PromMetric::new(
                    "node_network_transmit_bytes_total",
                    "Network device statistic transmit_bytes",
                    PromMetricType::Counter,
                    vec![
                        PromSample::new(
                            vec![PromLabel::new("device", "br0".to_string())],
                            1113957464f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth0".to_string())],
                            1781272596f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth1".to_string())],
                            220676085f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth2".to_string())],
                            3762007838f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "vlan1".to_string())],
                            2208073017f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "vlan2".to_string())],
                            3868443361f64,
                            None
                        ),
                    ]
                ),
            ]
        )
    }
}
