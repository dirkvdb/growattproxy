use chrono::SecondsFormat;
use rumqttc::Event::Incoming;
use rumqttc::{AsyncClient, Client, MqttOptions, Packet, QoS};
use std::time::Duration;

use crate::{
    dataprocessor::{FieldValue, GrowattData},
    ProxyError,
};

#[derive(Clone)]
pub struct MqttConfig {
    pub server: String,
    pub port: u16,
}

fn growatt_data_json(data: &GrowattData) -> String {
    use serde_json::{Map, Number, Value};

    let mut map = Map::new();

    for field in &data.fields {
        match &field.value {
            FieldValue::Text(str) => {
                map.insert(field.name.clone(), Value::String(str.clone()));
            }
            FieldValue::Date(date) => {
                map.insert(
                    field.name.clone(),
                    Value::String(date.to_rfc3339_opts(SecondsFormat::Secs, true)),
                );
            }
            FieldValue::Number(num) => {
                if *num.denom() == 1 {
                    map.insert(field.name.clone(), Value::Number(Number::from(*num.numer())));
                } else if let Some(val) = Number::from_f64(*num.numer() as f64 / *num.denom() as f64) {
                    map.insert(field.name.clone(), Value::Number(val));
                }
            }
        }
    }

    Value::Object(map).to_string()
}

pub async fn publish_data(data: &GrowattData, cfg: &MqttConfig) -> Result<(), ProxyError> {
    let mut mqttoptions = MqttOptions::new("growattproxy", cfg.server.as_str(), cfg.port);
    mqttoptions.set_keep_alive(Duration::from_secs(25));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .publish("energy/growattproxy", QoS::AtLeastOnce, false, growatt_data_json(&data))
        .await?;

    loop {
        let notification = eventloop.poll().await?;
        if let Incoming(msg) = notification {
            if let Packet::PubAck(_) = msg {
                break;
            }
        }
    }

    Ok(())
}

pub fn publish_data_sync(data: &GrowattData, cfg: &MqttConfig) -> Result<(), ProxyError> {
    let mut mqttoptions = MqttOptions::new("growattproxy", cfg.server.as_str(), cfg.port);
    mqttoptions.set_keep_alive(Duration::from_secs(25));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    client.publish("energy/growattproxy", QoS::AtLeastOnce, false, growatt_data_json(&data))?;

    // Wait for the ack
    for (_, notification) in connection.iter().enumerate() {
        if let Incoming(msg) = notification.unwrap() {
            if let Packet::PubAck(_) = msg {
                break;
            }
        }
    }

    Ok(())
}
