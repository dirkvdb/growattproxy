use rumqttc::{AsyncClient, Client, MqttOptions, QoS};
use std::time::Duration;

use crate::{
    dataprocessor::{FieldValue, GrowattData},
    ProxyError,
};

fn growatt_data_json(data: &GrowattData) -> String {
    use serde_json::{Map, Number, Value};

    let mut map = Map::new();

    for field in &data.fields {
        match &field.value {
            FieldValue::Text(str) => {
                map.insert(field.name.clone(), Value::String(str.clone()));
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

pub async fn publish_data(data: &GrowattData) -> Result<(), ProxyError> {
    let mut mqttoptions = MqttOptions::new("growattproxy", "192.168.1.13", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(25));

    let (client, _) = AsyncClient::new(mqttoptions, 10);
    client
        .publish("energy/growattproxy", QoS::AtLeastOnce, false, growatt_data_json(&data))
        .await?;

    Ok(())
}

pub fn publish_data_sync(data: &GrowattData) -> Result<(), ProxyError> {
    let mut mqttoptions = MqttOptions::new("growattproxy", "192.168.1.13", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(25));

    let (mut client, _) = Client::new(mqttoptions, 10);
    client.publish("energy/growattproxy", QoS::AtLeastOnce, false, growatt_data_json(&data))?;

    Ok(())
}
