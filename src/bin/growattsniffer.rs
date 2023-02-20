#![warn(clippy::unwrap_used)]
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "growwatproxy", about = "The growatt data upload proxy")]
struct Opt {
    // set the capture address to filter on
    #[clap(short = 'a', long = "addr")]
    addr: Option<String>,

    // set the port to filter on
    #[clap(short = 'p', long = "port", default_value_t = 5279)]
    port: u16,
}

fn main() {
    #[cfg(feature = "sniffer")]
    {
        use env_logger::{Env, TimestampPrecision};
        let opt = Opt::parse();

        env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
            .format_timestamp(Some(TimestampPrecision::Millis))
            .init();

        log::info!("Sniff sniff");
        growattproxy::sniffer::sniff(opt.addr.unwrap().as_str(), opt.port);
    }

    #[cfg(not(feature = "sniffer"))]
    println!("Sniffing support not enabled");
}
