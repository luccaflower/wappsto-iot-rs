use serde_json;
use std::fs::{read_to_string, DirBuilder, File};
use uuid::Uuid;

use crate::schema::Schema;

pub fn save(schema: Schema) {
    DirBuilder::new()
        .recursive(true)
        .create("network_instance")
        .unwrap();

    serde_json::to_writer(
        &File::create("network_instance/".to_owned() + &schema.meta.id.to_string() + ".json")
            .unwrap(),
        &schema,
    )
    .unwrap();
}
pub fn load(id: Uuid) -> Option<Schema> {
    println!("{}", &id);
    serde_json::from_str(
        &read_to_string("network_instance/".to_owned() + &id.to_string() + ".json")
            .expect("File not found"),
    )
    .unwrap()
}
