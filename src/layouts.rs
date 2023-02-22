use crate::dataprocessor::{FieldSpecification, LayoutSpecification};

pub fn t065004x() -> LayoutSpecification {
    LayoutSpecification::new(
        "T065004X",
        true,
        Vec::from([
            FieldSpecification::text("pvserial", 76, 10),
            FieldSpecification::date("date", 136),
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
    )
}

pub fn t06nnnnx() -> LayoutSpecification {
    LayoutSpecification::new(
        "t06NNNNX",
        true,
        Vec::from([
            FieldSpecification::text("pvserial", 76, 10),
            FieldSpecification::date("date", 136),
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
            FieldSpecification::number("pvenergytoday", 354, 4, 10),
            FieldSpecification::number("pvenergytotal", 362, 4, 10),
            FieldSpecification::number("pvtemperature", 530, 2, 10),
            FieldSpecification::number("pvipmtemperature", 534, 2, 10),
        ]),
    )
}

pub fn detect_layout(header: &[u8; 8]) -> LayoutSpecification {
    let mut layout = format!("T{:02x}{:02x}{:02x}", header[3], header[6], header[7]);
    let is_smart_meter = header[7] == 0x20 || header[7] == 0x1b;

    if is_smart_meter {
        layout.push('X');
    }

    match layout.as_str() {
        "T065004X" => t065004x(),
        _ => t06nnnnx(),
    }
}
