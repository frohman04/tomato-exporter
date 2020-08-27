use regex::Regex;

use crate::client::{Scraper, TomatoClientInternal};
use crate::prometheus::{PromLabel, PromMetric, PromMetricType, PromSample};

#[derive(Clone)]
pub struct UnameClient {
    client: TomatoClientInternal,
}

#[derive(Debug, PartialEq)]
struct Uname {
    domainname: String,
    machine: String,
    nodename: String,
    release: String,
    sysname: String,
    version: String,
}

impl UnameClient {
    pub fn new(client: TomatoClientInternal) -> UnameClient {
        UnameClient { client }
    }

    async fn get_uname(&self) -> Result<Uname, reqwest::Error> {
        let body = self.client.run_command("uname -a".to_string()).await?;
        Ok(UnameClient::parse_body(body))
    }

    fn parse_body(body: String) -> Uname {
        let uname_re = Regex::new(
            r"(?P<sysname>[a-zA-Z]+) (?P<nodename>[a-zA-Z0-9-_]+) (?P<release>[0-9.-a-z]+) (?P<version>.*) (?P<machine>[a-zA-Z0-9-_]+) ([a-zA-Z0-9]+)",
        )
        .unwrap();
        uname_re
            .captures(body.as_str())
            .map(|caps| Uname {
                domainname: "(none)".to_string(),
                machine: caps.name("machine").unwrap().as_str().to_string(),
                nodename: caps.name("nodename").unwrap().as_str().to_string(),
                release: caps.name("release").unwrap().as_str().to_string(),
                sysname: caps.name("sysname").unwrap().as_str().to_string(),
                version: caps.name("version").unwrap().as_str().to_string(),
            })
            .expect("Unable to parse uname data from command output")
    }

    fn raw_to_prom(uname: Uname) -> Vec<PromMetric> {
        vec![PromMetric::new(
            "node_uname_info",
            "Labeled system information as provided by the uname system call",
            PromMetricType::Gauge,
            vec![PromSample::new(
                vec![
                    PromLabel::new("domainname", uname.domainname),
                    PromLabel::new("machine", uname.machine),
                    PromLabel::new("nodename", uname.nodename),
                    PromLabel::new("release", uname.release),
                    PromLabel::new("sysname", uname.sysname),
                    PromLabel::new("version", uname.version),
                ],
                1f64,
                None,
            )],
        )]
    }
}

#[async_trait]
impl Scraper for UnameClient {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, reqwest::Error> {
        let raw_metrics = self.get_uname().await?;
        Ok(UnameClient::raw_to_prom(raw_metrics))
    }

    fn get_name(&self) -> String {
        "uname".to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_body() {
        assert_eq!(
            UnameClient::parse_body(
                "Linux karabor 2.6.22.19 #31 Thu Jul 16 01:30:27 CEST 2020 mips Tomato".to_string()
            ),
            Uname {
                domainname: "(none)".to_string(),
                machine: "mips".to_string(),
                nodename: "karabor".to_string(),
                release: "2.6.22.19".to_string(),
                sysname: "Linux".to_string(),
                version: "#31 Thu Jul 16 01:30:27 CEST 2020".to_string(),
            }
        )
    }

    #[test]
    fn test_raw_to_prom() {
        assert_eq!(
            UnameClient::raw_to_prom(Uname {
                domainname: "(none)".to_string(),
                machine: "mips".to_string(),
                nodename: "karabor".to_string(),
                release: "2.6.22.19".to_string(),
                sysname: "Linux".to_string(),
                version: "#31 Thu Jul 16 01:30:27 CEST 2020".to_string(),
            }),
            vec![PromMetric::new(
                "node_uname_info",
                "Labeled system information as provided by the uname system call",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    vec![
                        PromLabel::new("domainname", "(none)".to_string()),
                        PromLabel::new("machine", "mips".to_string()),
                        PromLabel::new("nodename", "karabor".to_string()),
                        PromLabel::new("release", "2.6.22.19".to_string()),
                        PromLabel::new("sysname", "Linux".to_string()),
                        PromLabel::new("version", "#31 Thu Jul 16 01:30:27 CEST 2020".to_string())
                    ],
                    1f64,
                    None
                )]
            )]
        )
    }
}
