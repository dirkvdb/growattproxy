use crate::{
    dataprocessor::{FieldValue, GrowattData},
    mqtt::{self, MqttConfig},
};

pub struct GrowattSnifferConfig {
    pub address: String,
    pub port: u16,
    pub mqtt: Option<MqttConfig>,
}

fn process_data(data: &GrowattData, cfg: &GrowattSnifferConfig, offset: u16) {
    log::info!(
        "[{}] valid growatt data buffered: {} [{} -> {}] ({})",
        data.packet_index(),
        data.is_buffered(),
        data.layout(),
        data.layout_spec,
        offset,
    );

    if !data.has_data() {
        return;
    }

    for field in &data.fields {
        match &field.value {
            FieldValue::Text(str) => {
                log::info!("{}: {}", field.name, str);
            }
            FieldValue::Date(date) => {
                log::info!(
                    "{}: {}",
                    field.name,
                    date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                );
            }
            FieldValue::Number(num) => {
                if *num.denom() == 1 {
                    log::info!("{}: {}", field.name, *num.numer());
                } else {
                    log::info!("{}: {}", field.name, *num.numer() as f64 / *num.denom() as f64);
                }
            }
        }
    }

    if (!data.is_buffered()) && cfg.mqtt.is_some() {
        if let Err(err) = mqtt::publish_data_sync(&data, cfg.mqtt.as_ref().unwrap()) {
            log::warn!("Failed to publish MQTT data: {err}");
        }
    }
}

pub fn sniff(cfg: &GrowattSnifferConfig) {
    let mut cap = pcap::Capture::from_device("any")
        .unwrap()
        .immediate_mode(true)
        .open()
        .unwrap();

    cap.filter(format!("host {} and tcp", cfg.address).as_str(), true)
        .unwrap();
    cap.filter(format!("dst port {}", cfg.port).as_str(), true).unwrap();

    let mut index = 1;
    while let Ok(packet) = cap.next_packet() {
        log::debug!("got packet: {} {}", packet.header.len, packet.data.len());
        if packet.data.len() > 128 {
            let mut data = Vec::from(packet.data);
            if let Ok(data) = GrowattData::from_buffer_auto_detect_layout(&mut data[56..]) {
                process_data(&data, cfg, 56);
            } else if let Ok(data) = GrowattData::from_buffer_auto_detect_layout(&mut data[68..]) {
                process_data(&data, cfg, 68);
            } else {
                log::warn!("invalid growatt data");
                crate::dump_packet(&packet.data, format!("growatt_invalid_{index}.bin").as_str());
                index += 1;
            }
        }
    }
}
