use std::{io::Write, path::Path};

use crate::{dataprocessor::GrowattData, layouts, mqtt};

fn dump_packet(data: &[u8], filename: &str) {
    let path = Path::new("/volume1/data/").join(filename);
    let mut file = std::fs::OpenOptions::new().write(true).create(true).open(path).unwrap();
    file.write_all(&data).unwrap();
}

pub fn sniff(address: &str, port: u16) {
    let mut cap = pcap::Capture::from_device("any")
        .unwrap()
        .immediate_mode(true)
        .open()
        .unwrap();

    cap.filter(format!("host {address} and tcp").as_str(), true).unwrap();
    cap.filter(format!("dst port {port}").as_str(), true).unwrap();

    let mut index = 1;
    while let Ok(packet) = cap.next_packet() {
        log::debug!("got packet: {} {}", packet.header.len, packet.data.len());
        if packet.data.len() > 128 {
            let mut data = Vec::from(packet.data);
            if let Ok(data) = GrowattData::from_buffer(&mut data[56..], &layouts::t065004x()) {
                log::info!("valid growatt data (56)");
                if let Err(err) = mqtt::publish_data_sync(&data) {
                    log::warn!("Failed to publish MQTT data: {err}");
                }
            } else if let Ok(data) = GrowattData::from_buffer(&mut data[68..], &layouts::t065004x()) {
                log::info!("valid growatt data (68)");
                if let Err(err) = mqtt::publish_data_sync(&data) {
                    log::warn!("Failed to publish MQTT data: {err}");
                }
            } else {
                log::warn!("invalid growatt data");
                dump_packet(&packet.data, format!("growatt_invalid_{index}.bin").as_str());
                index += 1;
            }
        }
    }
}
