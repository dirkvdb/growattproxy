use crate::{dataprocessor::GrowattData, layouts, mqtt};

pub fn sniff(address: &str) {
    let mut cap = pcap::Capture::from_device("any")
        .unwrap()
        .immediate_mode(true)
        .open()
        .unwrap();

    cap.filter(format!("host {address}").as_str(), true).unwrap();

    while let Ok(packet) = cap.next_packet() {
        if packet.data.len() > 128 {
            log::info!("got packet");
            let mut data = Vec::from(packet.data);
            if let Ok(data) = GrowattData::from_buffer(&mut data, &layouts::t065004x()) {
                log::info!("valid growatt data");
                if let Err(err) = mqtt::publish_data_sync(&data) {
                    log::warn!("Failed to publish MQTT data: {err}");
                }
            }
        }
    }
}
