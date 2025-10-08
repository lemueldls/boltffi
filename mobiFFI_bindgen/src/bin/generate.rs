use std::fs;
use std::path::PathBuf;

use mobiFFI_bindgen::model::*;
use mobiFFI_bindgen::Swift;

fn build_core_module() -> Module {
    let counter_class = Class::new("Counter")
        .with_constructor(Constructor::new())
        .with_method(
            Method::new("set", Receiver::RefMut)
                .with_param(Parameter::new("value", Type::Primitive(Primitive::U64))),
        )
        .with_method(Method::new("increment", Receiver::RefMut))
        .with_method(
            Method::new("get", Receiver::Ref)
                .with_output(Type::Primitive(Primitive::U64)),
        );

    let data_point = Record::new("DataPoint")
        .with_field(RecordField::new("x", Type::Primitive(Primitive::F64)))
        .with_field(RecordField::new("y", Type::Primitive(Primitive::F64)))
        .with_field(RecordField::new("timestamp", Type::Primitive(Primitive::I64)));

    let sensor_reading = Record::new("SensorReading")
        .with_field(RecordField::new("sensor_id", Type::Primitive(Primitive::I32)))
        .with_field(RecordField::new("timestamp_ms", Type::Primitive(Primitive::I64)))
        .with_field(RecordField::new("value", Type::Primitive(Primitive::F64)));

    let accumulator_class = Class::new("Accumulator")
        .with_constructor(Constructor::new())
        .with_method(
            Method::new("add", Receiver::RefMut)
                .with_param(Parameter::new("amount", Type::Primitive(Primitive::I64))),
        )
        .with_method(
            Method::new("get", Receiver::Ref)
                .with_output(Type::Primitive(Primitive::I64)),
        )
        .with_method(Method::new("reset", Receiver::RefMut));

    let sensor_monitor = Class::new("SensorMonitor")
        .with_constructor(Constructor::new())
        .with_method(
            Method::new("emit_reading", Receiver::RefMut)
                .with_param(Parameter::new("sensor_id", Type::Primitive(Primitive::I32)))
                .with_param(Parameter::new("timestamp_ms", Type::Primitive(Primitive::I64)))
                .with_param(Parameter::new("value", Type::Primitive(Primitive::F64))),
        )
        .with_method(
            Method::new("subscriber_count", Receiver::Ref)
                .with_output(Type::Primitive(Primitive::Usize)),
        )
        .with_stream(StreamMethod::new(
            "readings",
            Type::Record("SensorReading".into()),
        ));

    let direction_enum = Enumeration::new("Direction")
        .with_variant(Variant::new("north").with_discriminant(0))
        .with_variant(Variant::new("east").with_discriminant(1))
        .with_variant(Variant::new("south").with_discriminant(2))
        .with_variant(Variant::new("west").with_discriminant(3));

    Module::new("core")
        .with_class(counter_class)
        .with_class(accumulator_class)
        .with_class(sensor_monitor)
        .with_record(data_point)
        .with_record(sensor_reading)
        .with_enum(direction_enum)
}

fn generate_swift(module: &Module) -> String {
    let mut output = String::new();

    output.push_str("import Foundation\n\n");

    for class in &module.classes {
        output.push_str(&Swift::render_class(class, module));
        output.push_str("\n\n");
    }

    output
}

fn main() {
    let module = build_core_module();
    let swift_code = generate_swift(&module);

    let output_path = PathBuf::from("swift_test/Generated.swift");

    fs::write(&output_path, &swift_code).expect("Failed to write generated Swift file");

    println!("Generated Swift code written to: {}", output_path.display());
    println!("\n--- Preview ---\n");
    println!("{}", swift_code);
}
