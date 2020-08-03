extern crate regex;
extern crate reqwest;
extern crate serde_json;
extern crate serde_yaml;

#[cfg(test)]
#[macro_use]
extern crate maplit;

mod bandwidth;
mod config;

fn main() {
    let conf = config::load_conf();
    println!("{:?}", conf);
    let client = conf
        .modules
        .mod_bandwidth
        .map(|c| {
            bandwidth::BandwidthClient::new(
                c.router_ip,
                c.admin_username,
                c.admin_password,
                c.http_id,
            )
        })
        .expect("Must define mod_bandwidth configuration");
    println!("{:?}", client.get_bandwidth());
}
