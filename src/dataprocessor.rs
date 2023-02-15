use std::iter::zip;

use crc16::{State, MODBUS};

use crate::ProxyError;

const HEADER_SIZE: usize = 8;

pub enum FieldType {
    Text,
    Numeric,
}

pub struct FieldSpecification {
    name: String,
    field_type: FieldType,
    offset: usize,
    length: usize,
    divide: u16,
}

impl FieldSpecification {
    pub fn text(name: &str, offset: usize, length: usize) -> FieldSpecification {
        FieldSpecification {
            name: name.to_string(),
            field_type: FieldType::Text,
            offset: offset / 2,
            length,
            divide: 0,
        }
    }

    pub fn number(name: &str, offset: usize, length: usize, divide: u16) -> FieldSpecification {
        FieldSpecification {
            name: name.to_string(),
            field_type: FieldType::Numeric,
            offset: offset / 2,
            length,
            divide,
        }
    }
}

pub struct LayoutSpecification {
    decrypt: bool,
    fields: Vec<FieldSpecification>,
}

pub struct GrowattData {
    datalogserial: String,
    pvserial: String,
    // pvstatus: f64,
    // pvpowerin: f64,
    // pv1voltage: f64,
    // pv1current: f64,
    // pv1watt: f64,
    // pv2voltage: f64,
    // pv2current: f64,
    // pv2watt: f64,
    // pvpowerout: f64,
    // pvfrequentie: f64,
    // pvgridvoltage: f64,
    // pvgridcurrent: f64,
    // pvgridpower: f64,
    // pvgridvoltage2: f64,
    // pvgridcurrent2: f64,
    // pvgridpower2: f64,
    // pvgridvoltage3: f64,
    // pvgridcurrent3: f64,
    // pvgridpower3: f64,
    // totworktime: f64,
    // pvenergytoday: f64,
    // pvenergytotal: f64,
    // epvtotal: f64,
    // epv1today: f64,
    // epv1total: f64,
    // epv2today: f64,
    // epv2total: f64,
    // pvtemperature: f64,
    // pvipmtemperature: f64,
}

impl GrowattData {
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
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open("/Users/dirk/growatt.bin")?;

            std::io::Write::write_all(&mut file, &growatt_data);
        }

        for field in &spec.fields {
            let data_slice = &growatt_data[field.offset..field.offset + field.length];
            match field.field_type {
                FieldType::Text => {
                    log::info!(
                        "Text field: {} = {}",
                        field.name,
                        std::str::from_utf8(&data_slice)
                            .expect(format!("Invalid string: {:02x?}", data_slice).as_str())
                    );
                }

                FieldType::Numeric => {
                    if field.length == 2 {
                        log::info!(
                            "Numeric field: {} = {}",
                            field.name,
                            u16::from_be_bytes(data_slice.try_into().expect("Invalid u16 length"))
                        );
                    } else if field.length == 4 {
                        log::info!(
                            "Numeric field: {} = {}",
                            field.name,
                            u32::from_be_bytes(data_slice.try_into().expect("Invalid u32 length"))
                        );
                    }
                }
            }
        }

        Ok(GrowattData {
            datalogserial: String::from(""),
            pvserial: String::from(""),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::dataprocessor::FieldSpecification;
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
                FieldSpecification::number("pvstatus", 158, 2, 0),
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
        assert_eq!(gd.datalogserial, "serial");
    }
}
