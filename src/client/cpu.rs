use std::collections::BTreeMap;

use regex::Regex;

use crate::client::{DataClient, TomatoClientInternal};
use crate::prometheus::{PromLabel, PromMetric, PromMetricType, PromSample};

#[derive(Clone)]
pub struct CpuClient {
    client: TomatoClientInternal,
}

#[derive(Debug, PartialEq)]
struct CpuStats {
    user: f32,
    nice: f32,
    system: f32,
    idle: f32,
    iowait: Option<f32>,
    irq: Option<f32>,
    softirq: Option<f32>,
    steal: Option<f32>,
}

impl CpuClient {
    pub fn new(client: TomatoClientInternal) -> CpuClient {
        CpuClient { client }
    }

    async fn get_cpu(&self) -> Result<BTreeMap<u8, CpuStats>, reqwest::Error> {
        let body = self
            .client
            .run_command("cat /proc/stat".to_string())
            .await?;
        Ok(CpuClient::parse_body(body))
    }

    fn parse_body(body: String) -> BTreeMap<u8, CpuStats> {
        let cpu_re = Regex::new(r"cpu(?P<cpu>[0-9]+) (?P<jiffies>.*)").unwrap();
        cpu_re
            .captures_iter(body.as_str())
            .map(|raw_cpu| {
                let cpu_id = raw_cpu.name("cpu").unwrap().as_str().parse::<u8>().unwrap();
                let jiffies: Vec<u32> = raw_cpu
                    .name("jiffies")
                    .unwrap()
                    .as_str()
                    .split(" ")
                    .into_iter()
                    .map(|jif| jif.parse::<u32>().unwrap())
                    .collect();

                (
                    cpu_id,
                    CpuStats {
                        user: CpuClient::get_jiffie(&jiffies, 0),
                        nice: CpuClient::get_jiffie(&jiffies, 1),
                        system: CpuClient::get_jiffie(&jiffies, 2),
                        idle: CpuClient::get_jiffie(&jiffies, 3),
                        iowait: CpuClient::opt_jiffie(&jiffies, 4),
                        irq: CpuClient::opt_jiffie(&jiffies, 5),
                        softirq: CpuClient::opt_jiffie(&jiffies, 6),
                        steal: CpuClient::opt_jiffie(&jiffies, 7),
                    },
                )
            })
            .collect()
    }

    fn get_jiffie(jiffies: &Vec<u32>, i: usize) -> f32 {
        jiffies[i] as f32 / 100f32
    }

    fn opt_jiffie(jiffies: &Vec<u32>, i: usize) -> Option<f32> {
        if jiffies.len() > i {
            Some(jiffies[i] as f32 / 100f32)
        } else {
            None
        }
    }

    fn raw_to_prom(cpus: BTreeMap<u8, CpuStats>) -> Vec<PromMetric> {
        vec![PromMetric::new(
            "node_cpu_seconds_total",
            "Seconds the cpus spent in each mode",
            PromMetricType::Counter,
            cpus.into_iter()
                .map(|(i, cpu)| {
                    vec![
                        PromSample::new(
                            vec![
                                PromLabel::new("cpu", i.to_string()),
                                PromLabel::new("mode", "user".to_string()),
                            ],
                            cpu.user as f64,
                            None,
                        ),
                        PromSample::new(
                            vec![
                                PromLabel::new("cpu", i.to_string()),
                                PromLabel::new("mode", "nice".to_string()),
                            ],
                            cpu.nice as f64,
                            None,
                        ),
                        PromSample::new(
                            vec![
                                PromLabel::new("cpu", i.to_string()),
                                PromLabel::new("mode", "system".to_string()),
                            ],
                            cpu.system as f64,
                            None,
                        ),
                        PromSample::new(
                            vec![
                                PromLabel::new("cpu", i.to_string()),
                                PromLabel::new("mode", "idle".to_string()),
                            ],
                            cpu.idle as f64,
                            None,
                        ),
                    ]
                    .into_iter()
                    .chain(cpu.iowait.map_or_else(
                        || Vec::new(),
                        |iowait| {
                            vec![PromSample::new(
                                vec![
                                    PromLabel::new("cpu", i.to_string()),
                                    PromLabel::new("mode", "iowait".to_string()),
                                ],
                                iowait as f64,
                                None,
                            )]
                        },
                    ))
                    .chain(cpu.irq.map_or_else(
                        || Vec::new(),
                        |irq| {
                            vec![PromSample::new(
                                vec![
                                    PromLabel::new("cpu", i.to_string()),
                                    PromLabel::new("mode", "irq".to_string()),
                                ],
                                irq as f64,
                                None,
                            )]
                        },
                    ))
                    .chain(cpu.softirq.map_or_else(
                        || Vec::new(),
                        |softirq| {
                            vec![PromSample::new(
                                vec![
                                    PromLabel::new("cpu", i.to_string()),
                                    PromLabel::new("mode", "softirq".to_string()),
                                ],
                                softirq as f64,
                                None,
                            )]
                        },
                    ))
                    .chain(cpu.steal.map_or_else(
                        || Vec::new(),
                        |steal| {
                            vec![PromSample::new(
                                vec![
                                    PromLabel::new("cpu", i.to_string()),
                                    PromLabel::new("mode", "steal".to_string()),
                                ],
                                steal as f64,
                                None,
                            )]
                        },
                    ))
                    .collect::<Vec<PromSample>>()
                })
                .flatten()
                .collect(),
        )]
    }
}

#[async_trait]
impl DataClient for CpuClient {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, reqwest::Error> {
        let raw_metrics = self.get_cpu().await?;
        Ok(CpuClient::raw_to_prom(raw_metrics))
    }

    fn get_name(&self) -> String {
        "cpu".to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_body() {
        assert_eq!(
            CpuClient::parse_body(
                "cpu  162283 0 230563 168024492 2376 293698 4732481 0
cpu0 162283 0 230563 168024492 2376 293698 4732481 0
intr 846816216 0 0 0 203721765 315990752 153649036 8769 173445893 1 0 0 0 0 0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0 0 0 0
ctxt 15743031
btime 1596584154
processes 391097
procs_running 2
procs_blocked 0"
                    .to_string()
            ),
            btreemap!(0u8 => CpuStats {
                user: 162283 as f32 / 100f32,
                nice: 0f32,
                system: 230563 as f32 / 100f32,
                idle: 168024492 as f32 / 100f32,
                iowait: Some(2376 as f32 / 100f32),
                irq: Some(293698 as f32 / 100f32),
                softirq: Some(4732481 as f32 / 100f32),
                steal: Some(0f32),
            })
        )
    }

    #[test]
    fn test_raw_to_prom() {
        assert_eq!(
            CpuClient::raw_to_prom(btreemap!(0 => CpuStats {
                user: 162283 as f32 / 100f32,
                nice: 0f32,
                system: 230563 as f32 / 100f32,
                idle: 168024492 as f32 / 100f32,
                iowait: Some(2376 as f32 / 100f32),
                irq: Some(293698 as f32 / 100f32),
                softirq: Some(4732481 as f32 / 100f32),
                steal: Some(0f32),
            })),
            vec![PromMetric::new(
                "node_cpu_seconds_total",
                "Seconds the cpus spent in each mode",
                PromMetricType::Counter,
                vec![
                    PromSample::new(
                        vec![
                            PromLabel::new("cpu", "0".to_string()),
                            PromLabel::new("mode", "user".to_string()),
                        ],
                        (162283f32 / 100f32) as f64,
                        None
                    ),
                    PromSample::new(
                        vec![
                            PromLabel::new("cpu", "0".to_string()),
                            PromLabel::new("mode", "nice".to_string()),
                        ],
                        0f64,
                        None
                    ),
                    PromSample::new(
                        vec![
                            PromLabel::new("cpu", "0".to_string()),
                            PromLabel::new("mode", "system".to_string()),
                        ],
                        (230563f32 / 100f32) as f64,
                        None
                    ),
                    PromSample::new(
                        vec![
                            PromLabel::new("cpu", "0".to_string()),
                            PromLabel::new("mode", "idle".to_string()),
                        ],
                        (168024492f32 / 100f32) as f64,
                        None
                    ),
                    PromSample::new(
                        vec![
                            PromLabel::new("cpu", "0".to_string()),
                            PromLabel::new("mode", "iowait".to_string()),
                        ],
                        (2376f32 / 100f32) as f64,
                        None
                    ),
                    PromSample::new(
                        vec![
                            PromLabel::new("cpu", "0".to_string()),
                            PromLabel::new("mode", "irq".to_string()),
                        ],
                        (293698f32 / 100f32) as f64,
                        None
                    ),
                    PromSample::new(
                        vec![
                            PromLabel::new("cpu", "0".to_string()),
                            PromLabel::new("mode", "softirq".to_string()),
                        ],
                        (4732481f32 / 100f32) as f64,
                        None
                    ),
                    PromSample::new(
                        vec![
                            PromLabel::new("cpu", "0".to_string()),
                            PromLabel::new("mode", "steal".to_string()),
                        ],
                        0f64,
                        None
                    ),
                ]
            )]
        )
    }
}
