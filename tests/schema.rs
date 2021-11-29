use uuid::Uuid;
use wappsto_iot_rs::schema::{DeviceBuilder, SchemaBuilder, Value};
use wappsto_iot_rs::schema_store;

#[test]
fn saves_network_schema_to_data_store() {
    let id = Uuid::new_v4();

    let schema = SchemaBuilder::new(id)
        .named(String::from("test"))
        .add_device(
            DeviceBuilder::new()
                .named(String::from("buttom"))
                .add_value(Value::default())
                .create(),
        )
        .create();
    schema_store::save(schema);
    assert!(schema_store::load(id).is_ok())
}
