#![warn(clippy::unwrap_used)]
use clap::Parser;
use env_logger::{Env, TimestampPrecision};

#[derive(Parser, Debug)]
#[clap(name = "growwatproxy", about = "The growatt data upload proxy")]
struct Opt {
    // set the listen addr
    #[clap(short = 'a', long = "addr")]
    addr: Option<String>,
}

fn main() {
    let opt = Opt::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();

    #[cfg(feature = "sniffer")]
    {
        log::info!("Sniff sniff");
        growattproxy::sniffer::sniff(opt.addr.unwrap().as_str());
    }

    #[cfg(not(feature = "sniffer"))]
    println!("Sniffing support not enabled");
}
