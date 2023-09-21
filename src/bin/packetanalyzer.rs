#![warn(clippy::unwrap_used)]

use std::path::Path;

use clap::Parser;
use env_logger::{Env, TimestampPrecision};
use growattproxy::{dataprocessor::GrowattData, dump_packet, ProxyError};

#[derive(Parser, Debug)]
#[clap(name = "growwatproxy", about = "The growatt data upload proxy")]
struct Opt {
    // set the listen addr
    #[clap(short = 'i', long = "input")]
    input: String,

    #[clap(short = 's', long = "serial")]
    serial: Option<String>,

    #[clap(short = 'd', long = "decrypt", default_value_t = false)]
    decrypt: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ProxyError> {
    let opt = Opt::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();

    let mut data = std::fs::read(&opt.input)?;
    if opt.decrypt {
        GrowattData::decrypt_data(&mut data);
        dump_packet(&data, Path::new(format!("{}.decrypted", &opt.input).as_str()))?;
    } else {
        if let Err(err) = GrowattData::analyze_data(&mut data, opt.serial) {
            log::error!("Failed to analyze packet: {}", err);
        }
    }

    Ok(())
}
