use std::collections::HashMap;

use regex::Regex;
use reqwest::{Client, ClientBuilder};
use serde::{de::Error, Deserialize, Deserializer};

pub struct BandwidthClient {
    url: String,
    admin_username: String,
    admin_password: String,
    body: String,
    client: Client,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct BandwidthMeasurement {
    #[serde(deserialize_with = "from_hex")]
    rx: u64,
    #[serde(deserialize_with = "from_hex")]
    tx: u64,
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
    pub fn new(
        ip_address: String,
        admin_username: String,
        admin_password: String,
        http_id: String,
    ) -> BandwidthClient {
        info!("Pulling bandwidth data from {}", ip_address);
        BandwidthClient {
            url: format!("http://{}/update.cgi", ip_address),
            admin_username,
            admin_password,
            body: format!("exec=netdev&_http_id={}", http_id),
            client: ClientBuilder::new()
                .build()
                .expect("Unable to construct HTTP client"),
        }
    }

    pub async fn get_bandwidth(
        &self,
    ) -> Result<HashMap<String, BandwidthMeasurement>, reqwest::Error> {
        let response = self
            .client
            .post(&self.url.clone())
            .basic_auth(
                self.admin_username.clone(),
                Some(self.admin_password.clone()),
            )
            .body(self.body.clone())
            .send()
            .await?;
        let body = response.text().await?;
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
