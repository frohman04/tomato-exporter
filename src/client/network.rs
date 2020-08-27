use std::collections::BTreeMap;

use regex::{Captures, Regex};

use crate::client::{DataClient, TomatoClientInternal};
use crate::prometheus::{PromLabel, PromMetric, PromMetricType, PromSample};

#[derive(Clone)]
pub struct NetworkClient {
    client: TomatoClientInternal,
}

#[derive(Debug, PartialEq)]
struct NetworkInterface {
    pub name: String,
    pub rx_bytes: u64,
    pub rx_packets: u64,
    pub rx_errs: u64,
    pub rx_drop: u64,
    pub rx_fifo: u64,
    pub rx_frame: u64,
    pub rx_compressed: u64,
    pub rx_multicast: u64,
    pub tx_bytes: u64,
    pub tx_packets: u64,
    pub tx_errs: u64,
    pub tx_drop: u64,
    pub tx_fifo: u64,
    pub tx_colls: u64,
    pub tx_carrier: u64,
    pub tx_compressed: u64,
}

impl NetworkClient {
    pub fn new(client: TomatoClientInternal) -> NetworkClient {
        NetworkClient { client }
    }

    async fn get_network(&self) -> Result<BTreeMap<String, NetworkInterface>, reqwest::Error> {
        let body = self
            .client
            .run_command("cat /proc/net/dev".to_string())
            .await?;
        Ok(NetworkClient::parse_body(body))
    }

    fn parse_cap_u64(capture: &Captures, field: &str) -> u64 {
        capture
            .name(field)
            .unwrap()
            .as_str()
            .parse::<u64>()
            .unwrap()
    }

    fn parse_body(body: String) -> BTreeMap<String, NetworkInterface> {
        let if_re = Regex::new(r" *(?P<name>[a-z0-9]+): *(?P<rx_bytes>[0-9]+) +(?P<rx_packets>[0-9]+) +(?P<rx_errs>[0-9]+) +(?P<rx_drop>[0-9]+) +(?P<rx_fifo>[0-9]+) +(?P<rx_frame>[0-9]+) +(?P<rx_compressed>[0-9]+) +(?P<rx_multicast>[0-9]+) +(?P<tx_bytes>[0-9]+) +(?P<tx_packets>[0-9]+) +(?P<tx_errs>[0-9]+) +(?P<tx_drop>[0-9]+) +(?P<tx_fifo>[0-9]+) +(?P<tx_colls>[0-9]+) +(?P<tx_carrier>[0-9]+) +(?P<tx_compressed>[0-9]+)").unwrap();
        if_re
            .captures_iter(body.as_str())
            .map(|capture| {
                let name = capture.name("name").unwrap().as_str().to_string();
                (
                    name.clone(),
                    NetworkInterface {
                        name,
                        rx_bytes: NetworkClient::parse_cap_u64(&capture, "rx_bytes"),
                        rx_packets: NetworkClient::parse_cap_u64(&capture, "rx_packets"),
                        rx_errs: NetworkClient::parse_cap_u64(&capture, "rx_errs"),
                        rx_drop: NetworkClient::parse_cap_u64(&capture, "rx_drop"),
                        rx_fifo: NetworkClient::parse_cap_u64(&capture, "rx_fifo"),
                        rx_frame: NetworkClient::parse_cap_u64(&capture, "rx_frame"),
                        rx_compressed: NetworkClient::parse_cap_u64(&capture, "rx_compressed"),
                        rx_multicast: NetworkClient::parse_cap_u64(&capture, "rx_multicast"),
                        tx_bytes: NetworkClient::parse_cap_u64(&capture, "tx_bytes"),
                        tx_packets: NetworkClient::parse_cap_u64(&capture, "tx_packets"),
                        tx_errs: NetworkClient::parse_cap_u64(&capture, "tx_errs"),
                        tx_drop: NetworkClient::parse_cap_u64(&capture, "tx_drop"),
                        tx_fifo: NetworkClient::parse_cap_u64(&capture, "tx_fifo"),
                        tx_colls: NetworkClient::parse_cap_u64(&capture, "tx_colls"),
                        tx_carrier: NetworkClient::parse_cap_u64(&capture, "tx_carrier"),
                        tx_compressed: NetworkClient::parse_cap_u64(&capture, "tx_compressed"),
                    },
                )
            })
            .collect()
    }

    fn raw_to_prom(raw_metrics: BTreeMap<String, NetworkInterface>) -> Vec<PromMetric> {
        vec![
            PromMetric::new(
                "node_network_receive_bytes_total",
                "Network device statistic receive_bytes",
                PromMetricType::Counter,
                raw_metrics
                    .iter()
                    .filter_map(|(key, value)| {
                        let iface = value.to_owned();
                        if iface.rx_bytes > 0 {
                            Some(vec![PromSample::new(
                                vec![PromLabel::new("device", key.to_string())],
                                value.to_owned().rx_bytes as f64,
                                None,
                            )])
                        } else {
                            None
                        }
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
                    .filter_map(|(key, value)| {
                        let iface = value.to_owned();
                        if iface.tx_bytes > 0 {
                            Some(vec![PromSample::new(
                                vec![PromLabel::new("device", key.to_string())],
                                value.to_owned().tx_bytes as f64,
                                None,
                            )])
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect(),
            ),
        ]
    }
}

#[async_trait]
impl DataClient for NetworkClient {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, reqwest::Error> {
        let raw_metrics = self.get_network().await?;
        Ok(NetworkClient::raw_to_prom(raw_metrics))
    }

    fn get_name(&self) -> String {
        "network".to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl NetworkInterface {
        pub fn new(
            name: String,
            rx_bytes: u64,
            rx_packets: u64,
            rx_errs: u64,
            rx_drop: u64,
            rx_fifo: u64,
            rx_frame: u64,
            rx_compressed: u64,
            rx_multicast: u64,
            tx_bytes: u64,
            tx_packets: u64,
            tx_errs: u64,
            tx_drop: u64,
            tx_fifo: u64,
            tx_colls: u64,
            tx_carrier: u64,
            tx_compressed: u64,
        ) -> NetworkInterface {
            NetworkInterface {
                name,
                rx_bytes,
                rx_packets,
                rx_errs,
                rx_drop,
                rx_fifo,
                rx_frame,
                rx_compressed,
                rx_multicast,
                tx_bytes,
                tx_packets,
                tx_errs,
                tx_drop,
                tx_fifo,
                tx_colls,
                tx_carrier,
                tx_compressed,
            }
        }
    }

    #[test]
    fn test_parse_body() {
        let body = "Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop  fifo colls carrier compressed
    lo:   20551     116    0    0    0     0          0         0    20551     116    0    0    0     0       0          0
  eth0:1369176365 4125685    9    0    9     9          0         0 264555112  996099    0    0    0     0       0          0
 vlan1:38857540  128668    0    0    0     0          0      2820 114501528  166266    0    0    0     0       0          0
 vlan2:1256056495 3997017    0    0    0     0          0      3265 150053584  829833    0    0    0     0       0          0
  eth1:68892432  621865    0    0    0 139217          0         0 1040059644 3691882    9    0    0     0       0          0
  eth2:52613707  193305    0    0    0 148551          0         0 200476396  281861    7    0 0     0       0          0
   br0:141360332  899095    0    0    0     0          0     12878 1303031977 4051507    0    0    0     0       0          0
  imq0:       0       0    0    0    0     0          0         0        0       0    0    0    0     0       0          0
  imq1:       0       0    0    0    0     0          0         0        0       0    0    0    0     0       0          0";
        assert_eq!(
            NetworkClient::parse_body(body.to_string()),
            btreemap! {
                "lo".to_string() => NetworkInterface::new("lo".to_string(), 20551, 116, 0, 0, 0, 0, 0, 0, 20551, 116, 0, 0, 0, 0, 0, 0),
                "eth0".to_string() => NetworkInterface::new("eth0".to_string(), 1369176365, 4125685, 9, 0, 9, 9, 0, 0, 264555112, 996099, 0, 0, 0, 0, 0, 0),
                "eth1".to_string() => NetworkInterface::new("eth1".to_string(), 68892432, 621865, 0, 0, 0, 139217, 0, 0, 1040059644, 3691882, 9, 0, 0, 0, 0, 0),
                "eth2".to_string() => NetworkInterface::new("eth2".to_string(), 52613707, 193305, 0, 0, 0, 148551, 0, 0, 200476396, 281861, 7, 0, 0, 0, 0, 0),
                "vlan1".to_string() => NetworkInterface::new("vlan1".to_string(), 38857540, 128668, 0, 0, 0, 0, 0, 2820, 114501528, 166266, 0, 0, 0, 0, 0, 0),
                "vlan2".to_string() => NetworkInterface::new("vlan2".to_string(), 1256056495, 3997017, 0, 0, 0, 0, 0, 3265, 150053584, 829833, 0, 0, 0, 0, 0, 0),
                "br0".to_string() => NetworkInterface::new("br0".to_string(), 141360332, 899095, 0, 0, 0, 0, 0, 12878, 1303031977, 4051507, 0, 0, 0, 0, 0, 0),
                "imq0".to_string() => NetworkInterface::new("imq0".to_string(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
                "imq1".to_string() => NetworkInterface::new("imq1".to_string(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            }
        )
    }

    #[test]
    fn test_raw_to_prom() {
        assert_eq!(
            NetworkClient::raw_to_prom(btreemap! {
                "lo".to_string() => NetworkInterface::new("lo".to_string(), 20551, 116, 0, 0, 0, 0, 0, 0, 20551, 116, 0, 0, 0, 0, 0, 0),
                "eth0".to_string() => NetworkInterface::new("eth0".to_string(), 1369176365, 4125685, 9, 0, 9, 9, 0, 0, 264555112, 996099, 0, 0, 0, 0, 0, 0),
                "eth1".to_string() => NetworkInterface::new("eth1".to_string(), 68892432, 621865, 0, 0, 0, 139217, 0, 0, 1040059644, 3691882, 9, 0, 0, 0, 0, 0),
                "eth2".to_string() => NetworkInterface::new("eth2".to_string(), 52613707, 193305, 0, 0, 0, 148551, 0, 0, 200476396, 281861, 7, 0, 0, 0, 0, 0),
                "vlan1".to_string() => NetworkInterface::new("vlan1".to_string(), 38857540, 128668, 0, 0, 0, 0, 0, 2820, 114501528, 166266, 0, 0, 0, 0, 0, 0),
                "vlan2".to_string() => NetworkInterface::new("vlan2".to_string(), 1256056495, 3997017, 0, 0, 0, 0, 0, 3265, 150053584, 829833, 0, 0, 0, 0, 0, 0),
                "br0".to_string() => NetworkInterface::new("br0".to_string(), 141360332, 899095, 0, 0, 0, 0, 0, 12878, 1303031977, 4051507, 0, 0, 0, 0, 0, 0),
                "imq0".to_string() => NetworkInterface::new("imq0".to_string(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
                "imq1".to_string() => NetworkInterface::new("imq1".to_string(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            }),
            vec![
                PromMetric::new(
                    "node_network_receive_bytes_total",
                    "Network device statistic receive_bytes",
                    PromMetricType::Counter,
                    vec![
                        PromSample::new(
                            vec![PromLabel::new("device", "br0".to_string())],
                            141360332f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth0".to_string())],
                            1369176365f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth1".to_string())],
                            68892432f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth2".to_string())],
                            52613707f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "lo".to_string())],
                            20551f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "vlan1".to_string())],
                            38857540f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "vlan2".to_string())],
                            1256056495f64,
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
                            1303031977f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth0".to_string())],
                            264555112f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth1".to_string())],
                            1040059644f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "eth2".to_string())],
                            200476396f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "lo".to_string())],
                            20551f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "vlan1".to_string())],
                            114501528f64,
                            None
                        ),
                        PromSample::new(
                            vec![PromLabel::new("device", "vlan2".to_string())],
                            150053584f64,
                            None
                        ),
                    ]
                ),
            ]
        )
    }
}
