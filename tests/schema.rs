use uuid::Uuid;
use wappsto_iot_rs::fs_store::{self, load_schema};
use wappsto_iot_rs::schema::{DeviceBuilder, SchemaBuilder, Value};

#[test]
fn saves_network_schema_to_data_store() {
    let id = Uuid::new_v4();

    let schema = SchemaBuilder::new(id)
        .named(String::from("test"))
        .add_device(
            DeviceBuilder::new()
                .named(String::from("button"))
                .add_value(Value::default())
                .create(),
        )
        .create();
    fs_store::save_schema(schema);
    assert!(fs_store::load_schema(id).is_ok())
}
