use std::iter::zip;

use crc16::{State, MODBUS};
use num_rational::Rational64;

use crate::ProxyError;

const HEADER_SIZE: usize = 8;

pub enum FieldType {
    Text,
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
    Number(Rational64),
}

#[derive(Clone)]
pub struct Field {
    name: String,
    value: FieldValue,
}

impl Field {
    fn text(name: &str, value: &str) -> Field {
        Field {
            name: String::from(name),
            value: FieldValue::Text(String::from(value)),
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
    decrypt: bool,
    fields: Vec<FieldSpecification>,
}

pub struct GrowattData {
    fields: Vec<Field>,
}

impl GrowattData {
    fn new() -> GrowattData {
        GrowattData { fields: Vec::new() }
    }

    pub fn field_value(&self, name: &str) -> Option<FieldValue> {
        return self.fields.iter().find(|&f| f.name == name).map(|f| f.value.clone());
    }

    fn add_text_field(&mut self, name: &str, value: &str) {
        self.fields.push(Field::text(name, value));
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

    fn check_header(header: &[u8; HEADER_SIZE]) -> Result<(), ProxyError> {
        log::debug!("Header: {:02x?}", header);

        let is_smart_meter = header[7] == 0x20 || header[7] == 0x1b;
        log::debug!("Smart meter: {is_smart_meter}");

        log::debug!("Layout: T{:02x}{:02x}{:02x}", header[3], header[6], header[7]);

        Ok(())
    }

    pub fn from_buffer(growatt_data: &mut [u8], spec: &LayoutSpecification) -> Result<GrowattData, ProxyError> {
        if growatt_data.len() < 12 {
            // ACK message
            return Err(ProxyError::ParseError);
        }

        GrowattData::validate_integity(growatt_data)?;
        GrowattData::check_header(&growatt_data[0..8].try_into()?)?;

        if spec.decrypt {
            GrowattData::decrypt(growatt_data);
            // let mut file = std::fs::OpenOptions::new()
            //     .write(true)
            //     .create(true)
            //     .open("/Users/dirk/growatt.bin")?;

            // std::io::Write::write_all(&mut file, &growatt_data);
        }

        let mut result = GrowattData::new();

        for field in &spec.fields {
            let data_slice = &growatt_data[field.offset..field.offset + field.length];
            match field.field_type {
                FieldType::Text => {
                    let val = std::str::from_utf8(&data_slice)?;
                    result.add_text_field(field.name.as_str(), val);
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
    use crate::dataprocessor::FieldSpecification;
    use crate::dataprocessor::FieldValue;
    use crate::dataprocessor::LayoutSpecification;

    use super::GrowattData;

    fn init() {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }

    #[test]
    fn parse_data() {
        init();

        let t065004x = LayoutSpecification {
            decrypt: true,
            fields: Vec::from([
                FieldSpecification::text("pvserial", 76, 10),
                FieldSpecification::number("date", 136, 1, 10),
                FieldSpecification::number("pvstatus", 158, 2, 1),
                FieldSpecification::number("pvpowerin", 162, 4, 10),
                FieldSpecification::number("pv1voltage", 170, 2, 10),
                FieldSpecification::number("pv1current", 174, 2, 10),
                FieldSpecification::number("pv1watt", 178, 4, 10),
                FieldSpecification::number("pv2voltage", 186, 2, 10),
                FieldSpecification::number("pv2current", 190, 2, 10),
                FieldSpecification::number("pv2watt", 194, 4, 10),
                FieldSpecification::number("pvpowerout", 250, 4, 10),
                FieldSpecification::number("pvfrequentie", 258, 2, 100),
                FieldSpecification::number("pvgridvoltage", 262, 2, 10),
                FieldSpecification::number("pvgridcurrent", 266, 2, 10),
                FieldSpecification::number("pvgridpower", 270, 4, 10),
                FieldSpecification::number("pvgridvoltage2", 278, 2, 10),
                FieldSpecification::number("pvgridcurrent2", 282, 2, 10),
                FieldSpecification::number("pvgridpower2", 286, 4, 10),
                FieldSpecification::number("pvgridvoltage3", 294, 2, 10),
                FieldSpecification::number("pvgridcurrent3", 298, 2, 10),
                FieldSpecification::number("pvgridpower3", 302, 4, 10),
                FieldSpecification::number("totworktime", 346, 4, 7200),
                FieldSpecification::number("pvenergytoday", 354, 4, 10),
                FieldSpecification::number("pvenergytotal", 362, 4, 10),
                FieldSpecification::number("epvtotal", 370, 4, 10),
                FieldSpecification::number("epv1today", 378, 4, 10),
                FieldSpecification::number("epv1total", 386, 4, 10),
                FieldSpecification::number("epv2today", 394, 4, 10),
                FieldSpecification::number("epv2total", 402, 4, 10),
                FieldSpecification::number("pvtemperature", 530, 2, 10),
                FieldSpecification::number("pvipmtemperature", 534, 2, 10),
                FieldSpecification::number("pbusvolt", 550, 2, 1),
                FieldSpecification::number("nbusvolt", 554, 2, 1),
            ]),
        };

        let growatt_data = include_bytes!("./testdata/growatt_1.bin");
        let mut data = growatt_data.to_vec();

        let gd = GrowattData::from_buffer(&mut data, &t065004x).unwrap();
        assert_eq!(
            gd.field_value("pvserial").unwrap(),
            FieldValue::Text(String::from("MFK0CE306Q"))
        );
    }
}
