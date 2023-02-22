#![warn(clippy::unwrap_used)]
use clap::Parser;
use env_logger::{Env, TimestampPrecision};
use growattproxy::proxy::{self, GrowattProxyConfig};

#[derive(Parser, Debug)]
#[clap(name = "growwatproxy", about = "The growatt data upload proxy")]
struct Opt {
    // set the listen addr
    #[clap(
        short = 'a',
        long = "addr",
        env = "GP_LISTEN_ADDRESS",
        default_value = "0.0.0.0:5279"
    )]
    addr: String,

    // set the growatt addr
    #[clap(
        short = 'g',
        long = "growatt",
        env = "GP_GROWATT_ADDRESS",
        default_value = "47.91.67.66:5279"
    )]
    growatt_addr: String,

    // set the mqtt addr
    #[clap(long = "mqtt-addr", env = "GP_MQTT_ADDRESS")]
    mqtt_addr: Option<String>,

    #[clap(long = "mqtt-port", env = "GP_MQTT_PORT", default_value_t = 1883)]
    mqtt_port: u16,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let opt = Opt::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();

    let cfg = GrowattProxyConfig {
        listen_address: opt.addr,
        growatt_address: opt.growatt_addr,
        mqtt_address: opt.mqtt_addr,
        mqtt_port: opt.mqtt_port,
    };

    log::debug!("Run server on: {}", cfg.listen_address);
    proxy::GrowattProxy::new(cfg).run().await.expect("Failed to run proxy");
}
