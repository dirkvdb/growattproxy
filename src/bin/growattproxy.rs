#![warn(clippy::unwrap_used)]
use clap::Parser;
use env_logger::{Env, TimestampPrecision};
use growattproxy::proxy;

#[derive(Parser, Debug)]
#[clap(name = "growwatproxy", about = "The growatt data upload proxy")]
struct Opt {
    // set the listen addr
    #[clap(short = 'a', long = "addr")]
    addr: Option<String>,

    // set the growatt addr
    #[clap(short = 'g', long = "growatt")]
    growatt: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let opt = Opt::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();

    let proxy_addr = opt.addr.unwrap_or(String::from("0.0.0.0:5279"));
    let growatt_addr = opt.growatt.unwrap_or(String::from("47.91.67.66:5279"));
    log::debug!("Run server on: {proxy_addr}");

    proxy::GrowattProxy::new(proxy_addr.as_str(), growatt_addr.as_str()).run().await.expect("Failed to run proxy");

}
