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
pub fn load(id: Uuid) -> Result<Schema, LoadError> {
    let contents = match read_to_string("network_instance/".to_owned() + &id.to_string() + ".json")
    {
        Ok(s) => s,
        Err(_) => return Err(LoadError),
    };
    match serde_json::from_str(&contents) {
        Ok(s) => Ok(s),
        Err(_) => Err(LoadError),
    }
}

#[derive(Debug)]
pub struct LoadError;
