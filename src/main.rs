extern crate serde_yaml;

mod config;

fn main() {
    let conf = config::load_conf();
    println!("{:?}", conf);
}
