use uuid::Uuid;
use wappsto_iot_rs::schema::Schema;
use wappsto_iot_rs::schema_store;

#[test]
fn saves_network_schema_to_data_store() {
    let id = Uuid::new_v4();

    let schema = Schema::new(id);
    schema_store::save(schema);
    assert!(schema_store::load(id).is_some())
}
