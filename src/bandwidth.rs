use std::collections::HashMap;

use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer};

use crate::data_client::DataClient;
use crate::prometheus::{PromLabel, PromMetric, PromMetricType, PromSample};
use crate::tomato::TomatoClient;

#[derive(Clone)]
pub struct BandwidthClient {
    client: TomatoClient,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct BandwidthMeasurement {
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

    pub async fn get_bandwidth(
        &self,
    ) -> Result<HashMap<String, BandwidthMeasurement>, reqwest::Error> {
        let body = self
            .client
            .make_request(
                "update.cgi".to_string(),
                Some(hashmap! {"exec".to_string() => "netdev".to_string()}),
            )
            .await?;
        Ok(BandwidthClient::parse_body(body))
    }

    fn parse_body(body: String) -> HashMap<String, BandwidthMeasurement> {
        let regex = Regex::new(r"(0x[0-9a-fA-F]+)").unwrap();
        let cleaned = body
            .replace("netdev=", "")
            .replace(";", "")
            .replace("'", "\"")
            .replace("rx", "\"rx\"")
            .replace("tx", "\"tx\"");
        let cleaned = &*regex.replace_all(cleaned.as_str(), "\"$1\"");
        let parsed: HashMap<String, BandwidthMeasurement> =
            serde_json::from_str(cleaned).expect("Unable to parse response");
        parsed
    }
}

#[async_trait]
impl DataClient for BandwidthClient {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, reqwest::Error> {
        let raw_metrics = self.get_bandwidth().await?;
        Ok(vec![PromMetric::new(
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
        )])
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
        println!("{:?}", body);
        assert_eq!(
            BandwidthClient::parse_body(body.to_string()),
            hashmap! {
                "eth0".to_string() => BandwidthMeasurement { rx: 2876663457, tx: 1781272596 },
                "eth1".to_string() => BandwidthMeasurement { rx: 1091707937, tx: 220676085 },
                "eth2".to_string() => BandwidthMeasurement { rx: 1590928778, tx: 3762007838 },
                "vlan1".to_string() => BandwidthMeasurement { rx: 1280153509, tx: 2208073017 },
                "vlan2".to_string() => BandwidthMeasurement { rx: 590939678, tx: 3868443361 },
                "br0".to_string() => BandwidthMeasurement { rx: 3604816765, tx: 1113957464 },
            }
        )
    }
}
