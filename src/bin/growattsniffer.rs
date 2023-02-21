#![warn(clippy::unwrap_used)]
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "growwatproxy", about = "The growatt data upload proxy")]
struct Opt {
    // set the capture address to filter on
    #[clap(short = 'a', long = "addr", default_value = "0.0.0.0")]
    addr: String,

    // set the port to filter on
    #[clap(short = 'p', long = "port", default_value_t = 5279)]
    port: u16,

    // set the mqtt addr
    #[clap(long = "mqtt-addr")]
    mqtt_addr: Option<String>,

    #[clap(long = "mqtt-port", default_value_t = 1883)]
    mqtt_port: u16,
}

fn main() {
    #[cfg(feature = "sniffer")]
    {
        use env_logger::{Env, TimestampPrecision};
        use growattproxy::{mqtt::MqttConfig, sniffer::GrowattSnifferConfig};
        let opt = Opt::parse();

        env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
            .format_timestamp(Some(TimestampPrecision::Millis))
            .init();

        log::info!("Sniff sniff");
        let mqtt_config;
        if let Some(addr) = opt.mqtt_addr {
            mqtt_config = Some(MqttConfig {
                server: addr,
                port: opt.mqtt_port,
            });
        } else {
            mqtt_config = None;
        }

        growattproxy::sniffer::sniff(&GrowattSnifferConfig {
            address: opt.addr,
            port: opt.port,
            mqtt: mqtt_config,
        });
    }

    #[cfg(not(feature = "sniffer"))]
    println!("Sniffing support not enabled");
}
