use chrono::{DateTime, Utc};
use std::iter::zip;

use crc16::{State, MODBUS};
use num_rational::Rational64;

use crate::{layouts, ProxyError};

const HEADER_SIZE: usize = 8;

pub enum FieldType {
    Text,
    Date,
    Number(i64),
}

pub struct FieldSpecification {
    name: String,
    offset: usize,
    length: usize,
    field_type: FieldType,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FieldValue {
    Text(String),
    Date(DateTime<Utc>),
    Number(Rational64),
}

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub value: FieldValue,
}

impl Field {
    fn text(name: &str, value: &str) -> Field {
        Field {
            name: String::from(name),
            value: FieldValue::Text(String::from(value)),
        }
    }

    fn date(name: &str, date: DateTime<Utc>) -> Field {
        Field {
            name: String::from(name),
            value: FieldValue::Date(date),
        }
    }

    fn number(name: &str, value: Rational64) -> Field {
        Field {
            name: String::from(name),
            value: FieldValue::Number(value),
        }
    }
}

impl FieldSpecification {
    pub fn text(name: &str, offset: usize, length: usize) -> FieldSpecification {
        FieldSpecification {
            name: name.to_string(),
            offset: offset / 2,
            length,
            field_type: FieldType::Text,
        }
    }

    pub fn date(name: &str, offset: usize) -> FieldSpecification {
        FieldSpecification {
            name: name.to_string(),
            offset: offset / 2,
            length: 12,
            field_type: FieldType::Date,
        }
    }

    pub fn number(name: &str, offset: usize, length: usize, divide: i64) -> FieldSpecification {
        FieldSpecification {
            name: name.to_string(),
            offset: offset / 2,
            length,
            field_type: FieldType::Number(divide),
        }
    }
}

pub struct LayoutSpecification {
    id: String,
    decrypt: bool,
    fields: Vec<FieldSpecification>,
}

impl LayoutSpecification {
    pub fn new(id: &str, decrypt: bool, fields: Vec<FieldSpecification>) -> LayoutSpecification {
        LayoutSpecification {
            id: String::from(id),
            decrypt,
            fields,
        }
    }
}

pub struct GrowattData {
    pub header: [u8; HEADER_SIZE],
    pub layout_spec: String,
    pub fields: Vec<Field>,
}

impl GrowattData {
    fn new(layout_spec: &str) -> GrowattData {
        GrowattData {
            header: [0; HEADER_SIZE],
            layout_spec: String::from(layout_spec),
            fields: Vec::new(),
        }
    }

    pub fn packet_index(&self) -> u16 {
        u16::from_be_bytes(self.header[0..2].try_into().unwrap())
    }

    pub fn is_buffered(&self) -> bool {
        self.header[7] == 0x50
    }

    fn is_smart_meter(&self) -> bool {
        self.header[7] == 0x20 || self.header[7] == 0x1b
    }

    pub fn layout(&self) -> String {
        let mut layout = format!("T{:02x}{:02x}{:02x}", self.header[3], self.header[6], self.header[7]);
        if self.is_smart_meter() {
            layout.push('X');
        }

        layout
    }

    pub fn has_data(&self) -> bool {
        !self.fields.is_empty()
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn field_value(&self, name: &str) -> Option<FieldValue> {
        return self.fields.iter().find(|&f| f.name == name).map(|f| f.value.clone());
    }

    fn add_text_field(&mut self, name: &str, value: &str) {
        self.fields.push(Field::text(name, value));
    }

    fn add_date_field(&mut self, name: &str, date: DateTime<Utc>) {
        self.fields.push(Field::date(name, date));
    }

    fn add_number_field(&mut self, name: &str, value: Rational64) {
        self.fields.push(Field::number(name, value));
    }

    fn decrypt(growatt_data: &mut [u8]) {
        static MASK: &[u8; 7] = b"Growatt";

        // decrypt the data
        for (data_val, mask_val) in zip(growatt_data[8..].iter_mut(), MASK.iter().cycle()) {
            *data_val ^= *mask_val;
        }
    }

    fn validate_integity(data: &[u8]) -> Result<(), ProxyError> {
        let size = data.len();
        let header_payload_length = u16::from_be_bytes(data[4..6].try_into()?) as usize;
        let actual_payload_length = size - HEADER_SIZE;

        if header_payload_length != actual_payload_length {
            return Err(ProxyError::RuntimeError(format!(
                "Data payload size error: expected {header_payload_length} bytes got {actual_payload_length}"
            )));
        }

        let protocol = data[3];
        if protocol == 0x05 || protocol == 0x06 {
            let header_crc = u16::from_be_bytes(data[size - 2..].try_into()?);
            let actual_crc = State::<MODBUS>::calculate(&data[..size - 2]);

            if header_crc != actual_crc {
                return Err(ProxyError::RuntimeError(format!(
                    "Crc mismatch: expected {header_crc} got {actual_crc}"
                )));
            }

            log::debug!("CRC Matched!");
        }

        Ok(())
    }

    pub fn from_buffer_auto_detect_layout(growatt_data: &mut [u8]) -> Result<GrowattData, ProxyError> {
        if growatt_data.len() < 12 {
            // ACK message
            return Err(ProxyError::ParseError);
        }

        let spec = layouts::detect_layout(&growatt_data[0..8].try_into()?);
        let mut result = GrowattData::new(spec.id.as_str());
        result.header = growatt_data[0..8].try_into()?;

        GrowattData::validate_integity(growatt_data)?;

        if spec.decrypt {
            GrowattData::decrypt(growatt_data);
        }

        let layout = result.layout();
        if layout == "T065103" || layout == "T065129" {
            // ignore these layouts that do not contain power data
            return Ok(result);
        }

        for field in &spec.fields {
            if field.offset + field.length >= growatt_data.len() {
                log::debug!("Field '{}' out of range", field.name);
                continue;
            }

            let data_slice = &growatt_data[field.offset..field.offset + field.length];
            match field.field_type {
                FieldType::Text => {
                    let val = std::str::from_utf8(&data_slice)?;
                    result.add_text_field(field.name.as_str(), val);
                }

                FieldType::Date => {
                    //result.add_text_field(field.name.as_str(), val);
                    result.add_date_field(field.name.as_str(), Utc::now());
                }

                FieldType::Number(divide) => {
                    let val: i64;
                    if field.length == 1 {
                        val = u8::from_be_bytes(data_slice.try_into().expect("Invalid u8 length")) as i64;
                    } else if field.length == 2 {
                        val = u16::from_be_bytes(data_slice.try_into().expect("Invalid u16 length")) as i64;
                    } else if field.length == 4 {
                        val = u32::from_be_bytes(data_slice.try_into().expect("Invalid u32 length")) as i64;
                    } else {
                        return Err(ProxyError::RuntimeError(format!(
                            "Invalid length for number: {}",
                            field.length
                        )));
                    }

                    assert!(divide != 0);
                    result.add_number_field(field.name.as_str(), Rational64::new(val, divide));
                }
            }
        }

        Ok(result)
    }

    pub fn from_buffer(growatt_data: &mut [u8], spec: &LayoutSpecification) -> Result<GrowattData, ProxyError> {
        if growatt_data.len() < 12 {
            // ACK message
            return Err(ProxyError::ParseError);
        }

        let mut result = GrowattData::new(spec.id.as_str());
        result.header = growatt_data[0..8].try_into()?;

        GrowattData::validate_integity(growatt_data)?;

        if spec.decrypt {
            GrowattData::decrypt(growatt_data);
        }

        for field in &spec.fields {
            let data_slice = &growatt_data[field.offset..field.offset + field.length];
            match field.field_type {
                FieldType::Text => {
                    let val = std::str::from_utf8(&data_slice)?;
                    result.add_text_field(field.name.as_str(), val);
                }

                FieldType::Date => {
                    //result.add_text_field(field.name.as_str(), val);
                    result.add_date_field(field.name.as_str(), Utc::now());
                }

                FieldType::Number(divide) => {
                    let val: i64;
                    if field.length == 1 {
                        val = u8::from_be_bytes(data_slice.try_into().expect("Invalid u8 length")) as i64;
                    } else if field.length == 2 {
                        val = u16::from_be_bytes(data_slice.try_into().expect("Invalid u16 length")) as i64;
                    } else if field.length == 4 {
                        val = u32::from_be_bytes(data_slice.try_into().expect("Invalid u32 length")) as i64;
                    } else {
                        return Err(ProxyError::RuntimeError(format!(
                            "Invalid length for number: {}",
                            field.length
                        )));
                    }

                    assert!(divide != 0);
                    result.add_number_field(field.name.as_str(), Rational64::new(val, divide));
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use num_rational::Rational64;

    use crate::dataprocessor::FieldValue;
    use crate::layouts;

    use super::GrowattData;

    fn init() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }

    #[test]
    fn parse_data() {
        init();

        let growatt_data = include_bytes!("./testdata/growatt_1.bin");
        let mut data = growatt_data.to_vec();

        let gd = GrowattData::from_buffer(&mut data, &layouts::t065004x()).unwrap();
        assert_eq!(
            gd.field_value("pvserial").unwrap(),
            FieldValue::Text(String::from("MFK0CE306Q"))
        );

        assert_eq!(
            gd.field_value("pvstatus").unwrap(),
            FieldValue::Number(Rational64::from_integer(1))
        );

        assert_eq!(
            gd.field_value("pvpowerin").unwrap(),
            FieldValue::Number(Rational64::new(31326207, 10))
        );
    }
}
