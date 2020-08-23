use std::collections::HashMap;

use regex::Regex;
use serde::Deserialize;

use crate::client::{DataClient, TomatoClientInternal};
use crate::prometheus::{PromMetric, PromMetricType, PromSample};

#[derive(Clone)]
pub struct NodeClient {
    client: TomatoClientInternal,
}

#[derive(Debug, PartialEq)]
struct NodeMetrics {
    pub load_1m: f32,
    pub load_5m: f32,
    pub load_15m: f32,
    pub ram_total: u32,
    pub ram_free: u32,
    pub ram_buffer: u32,
    pub swap_total: u32,
    pub swap_free: u32,
    pub uptime: u64,
}

#[derive(Deserialize)]
struct SysInfo {
    pub uptime: u64,
    pub uptime_s: String,
    pub loads: Vec<u32>,
    pub totalram: u32,
    pub freeram: u32,
    pub bufferram: u32,
    pub cached: u32,
    pub totalswap: u32,
    pub freeswap: u32,
    pub totalfreeram: u32,
    pub procs: u32,
    pub flashsize: u32,
    pub systemtype: String,
    pub cpumodel: String,
    pub bogomips: String,
    pub cpuclk: String,
    pub cfeversion: String,
}

impl NodeClient {
    pub fn new(client: TomatoClientInternal) -> NodeClient {
        NodeClient { client }
    }

    async fn get_node(&self) -> Result<NodeMetrics, reqwest::Error> {
        let body = self
            .client
            .make_request("status-data.jsx".to_string(), Some(HashMap::new()))
            .await?;
        Ok(NodeClient::parse_body(body))
    }

    fn parse_body(body: String) -> NodeMetrics {
        let sysinfo_finder_re = Regex::new(r"sysinfo = \{(?s)([^}]+)};").unwrap();
        let sysinfo_raw = sysinfo_finder_re
            .find(body.as_str())
            .expect("Unable to find sysinfo in router response")
            .as_str()
            .replace("sysinfo = ", "")
            .replace(";", "")
            .replace("'", "\"");

        let key_fixer_re = Regex::new(r"(\s+)([$_a-zA-Z][$_a-zA-Z0-9]*):").unwrap();
        let sysinfo_json = &*key_fixer_re.replace_all(sysinfo_raw.as_str(), "$1\"$2\":");

        let sysinfo: SysInfo =
            serde_json::from_str(sysinfo_json).expect("Unable to parse response");

        NodeMetrics {
            load_1m: NodeClient::parse_load(sysinfo.loads[0]),
            load_5m: NodeClient::parse_load(sysinfo.loads[1]),
            load_15m: NodeClient::parse_load(sysinfo.loads[2]),
            ram_total: sysinfo.totalram,
            ram_buffer: sysinfo.bufferram,
            ram_free: sysinfo.freeram,
            swap_total: sysinfo.totalswap,
            swap_free: sysinfo.freeswap,
            uptime: sysinfo.uptime,
        }
    }

    fn parse_load(raw_load: u32) -> f32 {
        raw_load as f32 / 65536.0
    }

    fn raw_to_prom(raw_metrics: NodeMetrics) -> Vec<PromMetric> {
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
                "node_memory_MemTotal_bytes",
                "Memory information field MemTotal_bytes",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    Vec::new(),
                    raw_metrics.ram_total as f64,
                    None,
                )],
            ),
            PromMetric::new(
                "node_memory_Buffers_bytes",
                "Memory information field Buffers_bytes",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    Vec::new(),
                    raw_metrics.ram_buffer as f64,
                    None,
                )],
            ),
            PromMetric::new(
                "node_memory_MemFree_bytes",
                "Memory information field MemFree_bytes",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    Vec::new(),
                    raw_metrics.ram_free as f64,
                    None,
                )],
            ),
            PromMetric::new(
                "node_memory_SwapTotal_bytes",
                "Memory information field SwapTotal_bytes",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    Vec::new(),
                    raw_metrics.swap_total as f64,
                    None,
                )],
            ),
            PromMetric::new(
                "node_memory_SwapFree_bytes",
                "Memory information field SwapFree_bytes",
                PromMetricType::Gauge,
                vec![PromSample::new(
                    Vec::new(),
                    raw_metrics.swap_free as f64,
                    None,
                )],
            ),
            PromMetric::new(
                "node_time_seconds",
                "System time in seconds since epoch (1970)",
                PromMetricType::Gauge,
                vec![PromSample::new(Vec::new(), raw_metrics.uptime as f64, None)],
            ),
        ]
    }
}

#[async_trait]
impl DataClient for NodeClient {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, reqwest::Error> {
        let raw_metrics = self.get_node().await?;
        Ok(NodeClient::raw_to_prom(raw_metrics))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_body() {
        let body = "//
nvram = {
	'router_name': 'karabor',
	'wan_domain': 'home',
	'wan_hostname': 'karabor',
	'et0macaddr': 'E0:3F:49:05:BD:C0',
	'lan_proto': 'dhcp',
	'lan_ipaddr': '192.168.2.1',
	'dhcp_start': '2',
	'dhcp_num': '253',
	'dhcpd_startip': '192.168.2.2',
	'dhcpd_endip': '192.168.2.254',
	'lan_netmask': '255.255.255.0',
	'wl_security_mode': 'disabled',
	'wl0_security_mode': 'wpa2_personal',
	'wl1_security_mode': 'wpa2_personal',
	'wl_crypto': 'aes',
	'wl0_crypto': 'tkip+aes',
	'wl1_crypto': 'aes',
	'wl_mode': 'ap',
	'wl0_mode': 'ap',
	'wl1_mode': 'ap',
	'wl_wds_enable': '0',
	'wl0_wds_enable': '0',
	'wl1_wds_enable': '0',
	'wl_hwaddr': '',
	'wl0_hwaddr': 'E0:3F:49:05:BD:C0',
	'wl1_hwaddr': 'E0:3F:49:05:BD:0C',
	'wl_net_mode': 'mixed',
	'wl0_net_mode': 'mixed',
	'wl1_net_mode': 'mixed',
	'wl_radio': '1',
	'wl0_radio': '1',
	'wl1_radio': '1',
	'wl_channel': '6',
	'wl0_channel': '6',
	'wl1_channel': '56',
	'lan_gateway': '0.0.0.0',
	'wl_ssid': 'Tomato24',
	'wl0_ssid': 'karabor',
	'wl1_ssid': 'karabor5',
	'wl_closed': '0',
	'wl0_closed': '0',
	'wl1_closed': '0',
	't_model_name': 'Asus RT-N66U',
	't_features': '0xC1',
	'dhcp1_start': '',
	'dhcp1_num': '',
	'dhcpd1_startip': '',
	'dhcpd1_endip': '',
	'dhcp2_start': '',
	'dhcp2_num': '',
	'dhcpd2_startip': '',
	'dhcpd2_endip': '',
	'dhcp3_start': '',
	'dhcp3_num': '',
	'dhcpd3_startip': '',
	'dhcpd3_endip': '',
	'lan1_proto': '',
	'lan1_ipaddr': '',
	'lan1_netmask': '',
	'lan2_proto': '',
	'lan2_ipaddr': '',
	'lan2_netmask': '',
	'lan3_proto': '',
	'lan3_ipaddr': '',
	'lan3_netmask': '',
	'lan_ifname': 'br0',
	'lan1_ifname': '',
	'lan2_ifname': '',
	'lan3_ifname': '',
	'lan_ifnames': 'vlan1 eth1 eth2',
	'lan1_ifnames': '',
	'lan2_ifnames': '',
	'lan3_ifnames': '',
	'wan_ifnames': 'vlan2',
	'tomatoanon_enable': '1',
	'tomatoanon_answer': '1',
	'lan_desc': '1',
	'wan_ppp_get_ip': '0.0.0.0',
	'wan_pptp_dhcp': '1',
	'wan_pptp_server_ip': '',
	'wan_ipaddr_buf': '',
	'wan_gateway': '192.168.1.1',
	'wan_gateway_get': '0.0.0.0',
	'wan_get_domain': 'fios-router.home',
	'wan_hwaddr': 'E0:3F:49:05:BD:C1',
	'wan_ipaddr': '192.168.1.2',
	'wan_netmask': '255.255.255.0',
	'wan_proto': 'dhcp',
	'wan_run_mtu': '1500',
	'wan_sta': '',
	'wan2_ppp_get_ip': '',
	'wan2_pptp_dhcp': '1',
	'wan2_pptp_server_ip': '',
	'wan2_ipaddr_buf': '',
	'wan2_gateway': '0.0.0.0',
	'wan2_gateway_get': '0.0.0.0',
	'wan2_get_domain': '',
	'wan2_hwaddr': '',
	'wan2_ipaddr': '0.0.0.0',
	'wan2_netmask': '0.0.0.0',
	'wan2_proto': 'dhcp',
	'wan2_run_mtu': '',
	'wan2_sta': '',
	'wan3_ppp_get_ip': '',
	'wan3_pptp_dhcp': '1',
	'wan3_pptp_server_ip': '',
	'wan3_ipaddr_buf': '',
	'wan3_gateway': '0.0.0.0',
	'wan3_gateway_get': '0.0.0.0',
	'wan3_get_domain': '',
	'wan3_hwaddr': '',
	'wan3_ipaddr': '0.0.0.0',
	'wan3_netmask': '0.0.0.0',
	'wan3_proto': 'dhcp',
	'wan3_run_mtu': '',
	'wan3_sta': '',
	'wan4_ppp_get_ip': '',
	'wan4_pptp_dhcp': '1',
	'wan4_pptp_server_ip': '',
	'wan4_ipaddr_buf': '',
	'wan4_gateway': '0.0.0.0',
	'wan4_gateway_get': '0.0.0.0',
	'wan4_get_domain': '',
	'wan4_hwaddr': '',
	'wan4_ipaddr': '0.0.0.0',
	'wan4_netmask': '0.0.0.0',
	'wan4_proto': 'dhcp',
	'wan4_run_mtu': '',
	'wan4_sta': '',
	'mwan_num': '1',
	'pptp_client_enable': '0',
	'pptp_client_ipaddr': '',
	'pptp_client_netmask': '',
	'pptp_client_gateway': '',
	'pptp_client_get_dns': '',
	'pptp_client_srvsub': '10.0.0.0',
	'pptp_client_srvsubmsk': '255.0.0.0',
	'wan_modem_type': '',
	'wan2_modem_type': '',
	'wan3_modem_type': '',
	'wan4_modem_type': '',
	'wan_hilink_ip': '0.0.0.0',
	'wan2_hilink_ip': '0.0.0.0',
	'wan3_hilink_ip': '0.0.0.0',
	'wan4_hilink_ip': '0.0.0.0',
	'wan_status_script': '0',
	'wan2_status_script': '0',
	'wan3_status_script': '0',
	'wan4_status_script': '0',
	'wl_unit': '0',
	'http_id': 'TID4bad0f0eba40bd0c',
	'web_mx': 'status,bwm',
	'web_pb': ''};

//
//
sysinfo = {
	uptime: 1391983,
	uptime_s: '16 days, 02:39:43',
	loads: [224, 2400, 0],
	totalram: 261836800,
	freeram: 227065856,
	bufferram: 5394432,
	cached: 15699968,
	totalswap: 0,
	freeswap: 0,
	totalfreeram: 248160256,
	procs: 35,
	flashsize: 32,
	systemtype: 'Broadcom BCM5300 chip rev 1 pkg 0',
	cpumodel: 'MIPS 74K V4.9',
	bogomips: '299.82',
	cpuclk: '600',
	cfeversion: '1.0.1.4'};

//
wlstats = [ { radio: 1, client: 0, channel:  6, mhz: 2437, rate: 234, ctrlsb: 'none', nbw: 20, rssi: 0, noise: -99, intf: 0 }
,{ radio: 1, client: 0, channel:  56, mhz: 5280, rate: 300, ctrlsb: 'upper', nbw: 40, rssi: 0, noise: -99, intf: 0 }
];

stats = { };
do {
var a, b, i;
var xifs = ['wan', 'lan', 'lan1', 'lan2', 'lan3', 'wan2', 'wan3', 'wan4'];
stats.anon_enable = nvram.tomatoanon_enable;
stats.anon_answer = nvram.tomatoanon_answer;
stats.lan_desc = nvram.lan_desc;
if (typeof(last_wan_proto) == 'undefined') {
last_wan_proto = nvram.wan_proto;
}
else if (last_wan_proto != nvram.wan_proto) {
reloadPage();
}
stats.flashsize = sysinfo.flashsize+'MB';
stats.cpumhz = sysinfo.cpuclk+'MHz';
stats.cputemp = sysinfo.cputemp+'°';
stats.systemtype = sysinfo.systemtype;
stats.cfeversion = sysinfo.cfeversion;
stats.cpuload = ((sysinfo.loads[0] / 65536.0).toFixed(2) + '<small> / </small> ' + (sysinfo.loads[1] / 65536.0).toFixed(2) + '<small> / </small>' + (sysinfo.loads[2] / 65536.0).toFixed(2));
stats.uptime = sysinfo.uptime_s;
a = sysinfo.totalram;
b = sysinfo.totalfreeram;
stats.memory = scaleSize(a) + ' / ' + scaleSize(b) + ' <small>(' + (b / a * 100.0).toFixed(2) + '%)</small>';
if (sysinfo.totalswap > 0) {
a = sysinfo.totalswap;
b = sysinfo.freeswap;
stats.swap = scaleSize(a) + ' / ' + scaleSize(b) + ' <small>(' + (b / a * 100.0).toFixed(2) + '%)</small>';
} else
stats.swap = '';
stats.time = 'Thu, 20 Aug 2020 22:15:37 -0400';
stats.wanup = [1,0,0,0];
stats.wanuptime = ['16 days, 02:39:11','-','-','-'];
stats.wanlease = ['0 days, 21:28:28','0 days, 00:00:00','0 days, 00:00:00','0 days, 00:00:00'];
stats.dns = [['192.168.2.135:53','1.1.1.1:53'],[],[],[]];
stats.wanip = [];
stats.wannetmask = [];
stats.wangateway = [];
for (var uidx = 1; uidx <= nvram.mwan_num; ++uidx) {
var u = (uidx>1) ? uidx : '';
stats.wanip[uidx-1] = nvram['wan'+u+'_ipaddr'];
stats.wannetmask[uidx-1] = nvram['wan'+u+'_netmask'];
stats.wangateway[uidx-1] = nvram['wan'+u+'_gateway_get'];
if (stats.wangateway[uidx-1] == '0.0.0.0' || stats.wangateway[uidx-1] == '')
stats.wangateway[uidx-1] = nvram['wan'+u+'_gateway'];
switch (nvram['wan'+u+'_proto']) {
case 'pptp':
case 'l2tp':
case 'pppoe':
if (stats.wanup[uidx-1]) {
stats.wanip[uidx-1] = nvram['wan'+u+'_ppp_get_ip'];
if (nvram['wan'+u+'_pptp_dhcp'] == '1') {
if (nvram['wan'+u+'_ipaddr'] != '' && nvram['wan'+u+'_ipaddr'] != '0.0.0.0' && nvram['wan'+u+'_ipaddr'] != stats.wanip[uidx-1])
stats.wanip[uidx-1] += '&nbsp;&nbsp;<small>(DHCP: ' + nvram['wan'+u+'_ipaddr'] + ')</small>';
if (nvram['wan'+u+'_gateway'] != '' && nvram['wan'+u+'_gateway']  != '0.0.0.0' && nvram['wan'+u+'_gateway']  != stats.wangateway[uidx-1])
stats.wangateway[uidx-1] += '&nbsp;&nbsp;<small>(DHCP: ' + nvram['wan'+u+'_gateway']  + ')</small>';
}
if (stats.wannetmask[uidx-1] == '0.0.0.0')
stats.wannetmask[uidx-1] = '255.255.255.255';
}
else {
if (nvram['wan'+u+'_proto'] == 'pptp')
stats.wangateway[uidx-1] = nvram['wan'+u+'_pptp_server_ip'];
}
break;
default:
if (!stats.wanup[uidx-1]) {
stats.wanip[uidx-1] = '0.0.0.0';
stats.wannetmask[uidx-1] = '0.0.0.0';
stats.wangateway[uidx-1] = '0.0.0.0';
}
}
}
stats.ip6_wan = ((typeof(sysinfo.ip6_wan) != 'undefined') ? sysinfo.ip6_wan : '') + '';
stats.ip6_lan = ((typeof(sysinfo.ip6_lan) != 'undefined') ? sysinfo.ip6_lan : '') + '';
stats.ip6_lan_ll = ((typeof(sysinfo.ip6_lan_ll) != 'undefined') ? sysinfo.ip6_lan_ll : '') + '';
stats.ip6_lan1 = ((typeof(sysinfo.ip6_lan1) != 'undefined') ? sysinfo.ip6_lan1 : '') + '';
stats.ip6_lan1_ll = ((typeof(sysinfo.ip6_lan1_ll) != 'undefined') ? sysinfo.ip6_lan1_ll : '') + '';
stats.ip6_lan2 = ((typeof(sysinfo.ip6_lan2) != 'undefined') ? sysinfo.ip6_lan2 : '') + '';
stats.ip6_lan2_ll = ((typeof(sysinfo.ip6_lan2_ll) != 'undefined') ? sysinfo.ip6_lan2_ll : '') + '';
stats.ip6_lan3 = ((typeof(sysinfo.ip6_lan3) != 'undefined') ? sysinfo.ip6_lan3 : '') + '';
stats.ip6_lan3_ll = ((typeof(sysinfo.ip6_lan3_ll) != 'undefined') ? sysinfo.ip6_lan3_ll : '') + '';
stats.wanstatus = ['Connected','Disconnected','Disconnected','Disconnected'];
for (var uidx = 1; uidx <= nvram.mwan_num; ++uidx) {
if (stats.wanstatus[uidx-1] != 'Connected') stats.wanstatus[uidx-1] = '<b>' + stats.wanstatus[uidx-1] + '</b>';
}
stats.channel = [];
stats.interference = [];
stats.qual = [];
for (var uidx = 0; uidx < wl_ifaces.length; ++uidx) {
u = wl_unit(uidx);
a = i = wlstats[uidx].channel * 1;
if (i < 0) i = -i;
stats.channel.push('<a href=\"tools-survey.asp\">' + ((i) ? i + '' : 'Auto') +
((wlstats[uidx].mhz) ? ' - ' + (wlstats[uidx].mhz / 1000.0).toFixed(3) + ' <small>GHz</small>' : '') + '</a>' +
((a < 0) ? ' <small>(scanning...)</small>' : ''));
stats.interference.push((wlstats[uidx].intf >= 0) ? ((wlstats[uidx].intf) ? 'Severe' : 'Acceptable') : '');
a = wlstats[uidx].nbw * 1;
wlstats[uidx].nbw = (a > 0) ? (a + ' <small>MHz</small>') : 'Auto';
if (wlstats[uidx].radio) {
a = wlstats[uidx].rate * 1;
if (a > 0)
wlstats[uidx].rate = Math.floor(a / 2) + ((a & 1) ? '.5' : '') + ' <small>Mbps</small>';
else
wlstats[uidx].rate = '-';
if (wlstats[uidx].client) {
if (wlstats[uidx].rssi == 0) a = 0;
else a = MAX(wlstats[uidx].rssi - wlstats[uidx].noise, 0);
stats.qual.push(a + ' <img src=\"bar' + MIN(MAX(Math.floor(a / 10), 1), 6) + '.gif\">');
}
else {
stats.qual.push('');
}
wlstats[uidx].noise += ' <small>dBm</small>';
wlstats[uidx].rssi += ' <small>dBm</small>';
}
else {
wlstats[uidx].rate = '';
wlstats[uidx].noise = '';
wlstats[uidx].rssi = '';
stats.qual.push('');
}
if (wl_ifaces[uidx][6] != 1) {
wlstats[uidx].ifstatus = '<b>Down</b>';
}
else {
wlstats[uidx].ifstatus = 'Up';
for (i = 0; i < xifs.length ; ++i) {
if ((nvram[xifs[i] + '_ifnames']).indexOf(wl_ifaces[uidx][0]) >= 0) {
wlstats[uidx].ifstatus = wlstats[uidx].ifstatus + ' (' + xifs[i].toUpperCase() + ')';
break;
}
}
}
}
} while (0);
";
        assert_eq!(
            NodeClient::parse_body(body.to_string()),
            NodeMetrics {
                load_1m: 224 as f32 / 65536.0,
                load_5m: 2400 as f32 / 65536.0,
                load_15m: 0 as f32 / 65536.0,
                ram_total: 261836800,
                ram_buffer: 5394432,
                ram_free: 227065856,
                swap_total: 0,
                swap_free: 0,
                uptime: 1391983,
            }
        )
    }

    #[test]
    fn test_raw_to_prom() {
        assert_eq!(
            NodeClient::raw_to_prom(NodeMetrics {
                load_1m: 224 as f32 / 65536.0,
                load_5m: 2400 as f32 / 65536.0,
                load_15m: 0 as f32 / 65536.0,
                ram_total: 261836800,
                ram_buffer: 5394432,
                ram_free: 227065856,
                swap_total: 0,
                swap_free: 0,
                uptime: 1391983,
            }),
            vec![
                PromMetric::new(
                    "node_load1",
                    "1m load average",
                    PromMetricType::Gauge,
                    vec![PromSample::new(
                        Vec::new(),
                        (224 as f32 / 65536.0) as f64,
                        None,
                    )],
                ),
                PromMetric::new(
                    "node_load5",
                    "5m load average",
                    PromMetricType::Gauge,
                    vec![PromSample::new(
                        Vec::new(),
                        (2400 as f32 / 65536.0) as f64,
                        None,
                    )],
                ),
                PromMetric::new(
                    "node_load15",
                    "15m load average",
                    PromMetricType::Gauge,
                    vec![PromSample::new(
                        Vec::new(),
                        (0 as f32 / 65536.0) as f64,
                        None,
                    )],
                ),
                PromMetric::new(
                    "node_memory_MemTotal_bytes",
                    "Memory information field MemTotal_bytes",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), 261836800 as f64, None,)],
                ),
                PromMetric::new(
                    "node_memory_Buffers_bytes",
                    "Memory information field Buffers_bytes",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), 5394432 as f64, None,)],
                ),
                PromMetric::new(
                    "node_memory_MemFree_bytes",
                    "Memory information field MemFree_bytes",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), 227065856 as f64, None,)],
                ),
                PromMetric::new(
                    "node_memory_SwapTotal_bytes",
                    "Memory information field SwapTotal_bytes",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), 0 as f64, None,)],
                ),
                PromMetric::new(
                    "node_memory_SwapFree_bytes",
                    "Memory information field SwapFree_bytes",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), 0 as f64, None,)],
                ),
                PromMetric::new(
                    "node_time_seconds",
                    "System time in seconds since epoch (1970)",
                    PromMetricType::Gauge,
                    vec![PromSample::new(Vec::new(), 1391983 as f64, None)],
                ),
            ]
        )
    }
}
