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

pub fn field_value_to_json_value(val: &FieldValue, factor: Option<f64>) -> Option<serde_json::Value> {
    use serde_json::{Number, Value};

    match val {
        FieldValue::Text(str) => {
            return Some(Value::String(str.clone()));
        }
        FieldValue::Date(date) => {
            return Some(Value::String(date.to_rfc3339_opts(SecondsFormat::Secs, true)));
        }
        FieldValue::Number(num) => {
            if *num.denom() == 1 {
                if let Some(factor) = factor {
                    if let Some(val) = Number::from_f64(*num.numer() as f64 * factor) {
                        return Some(Value::Number(val));
                    }
                } else {
                    return Some(Value::Number(Number::from(*num.numer())));
                }
            } else if let Some(val) =
                Number::from_f64((*num.numer() as f64 / *num.denom() as f64) * factor.unwrap_or(1.0))
            {
                return Some(Value::Number(val));
            }
        }
    }

    None
}

fn growatt_data_json_remi(data: &GrowattData) -> String {
    use serde_json::{Map, Number, Value};

    let mut map = Map::new();

    map.insert(String::from("ident"), Value::String(String::from("pvpanelendak")));
    map.insert(String::from("device_CH"), Value::Number(Number::from(1)));
    map.insert(String::from("Name"), Value::String(String::from("PV")));
    map.insert(String::from("CHname"), Value::String(String::from("PV")));
    map.insert(String::from("Type"), Value::String(String::from("MB")));
    map.insert(String::from("Units"), Value::String(String::from("kWh")));

    if let Some(field) = data.field_value("pvgridvoltage") {
        if let Some(field_val) = field_value_to_json_value(&field, None) {
            map.insert(String::from("U"), field_val);
        }
    }

    if let Some(field) = data.field_value("pvgridcurrent") {
        if let Some(field_val) = field_value_to_json_value(&field, Some(1000.0)) {
            map.insert(String::from("I"), field_val);
        }
    }

    if let Some(field) = data.field_value("pvpowerout") {
        if let Some(field_val) = field_value_to_json_value(&field, None) {
            map.insert(String::from("P"), field_val);
        }
    }

    map.insert(String::from("HC"), Value::Number(Number::from(0)));
    map.insert(String::from("DC"), Value::Number(Number::from(0)));
    map.insert(String::from("MC"), Value::Number(Number::from(0)));
    map.insert(String::from("CH"), Value::Number(Number::from(0)));
    map.insert(String::from("CL"), Value::Number(Number::from(0)));

    if let Some(field) = data.field_value("pvenergytoday") {
        if let Some(field_val) = field_value_to_json_value(&field, Some(1000.0)) {
            map.insert(String::from("DC"), field_val);
        }
    }

    if let Some(field) = data.field_value("pvenergytotal") {
        if let Some(field_val) = field_value_to_json_value(&field, Some(1000.0)) {
            map.insert(String::from("CH"), field_val);
        }
    }

    Value::Object(map).to_string()
}

fn growatt_data_json(data: &GrowattData) -> String {
    use serde_json::{Map, Value};

    let mut map = Map::new();

    for field in &data.fields {
        if let Some(field_val) = field_value_to_json_value(&field.value, None) {
            map.insert(field.name.clone(), field_val);
        }
    }

    Value::Object(map).to_string()
}

pub async fn publish_data(data: &GrowattData, cfg: &MqttConfig) -> Result<(), ProxyError> {
    let mut mqttoptions = MqttOptions::new("growattproxy", cfg.server.as_str(), cfg.port);
    mqttoptions.set_keep_alive(Duration::from_secs(25));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .publish(
            "pvpanelendak/PUB/CH1",
            QoS::AtLeastOnce,
            false,
            growatt_data_json_remi(&data),
        )
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
