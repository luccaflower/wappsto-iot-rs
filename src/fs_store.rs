use serde_json;
use std::error::Error;
use std::fs::{read_to_string, write, DirBuilder, File};
use uuid::Uuid;

use crate::create_network::{Creator, CreatorNetwork};
use crate::schema::Schema;

///Save network schema to data store
pub fn save_schema(schema: Schema) {
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

///Load network schema from data store
pub fn load_schema(id: Uuid) -> Result<Schema, Box<dyn Error>> {
    let contents = match read_to_string("network_instance/".to_owned() + &id.to_string() + ".json")
    {
        Ok(s) => s,
        Err(e) => return Err(Box::new(e)),
    };
    match serde_json::from_str(&contents) {
        Ok(s) => Ok(s),
        Err(e) => Err(Box::new(e)),
    }
}

///Save the creator object into certificates
pub fn save_certs(creator: Creator) -> Result<(), Box<dyn Error>> {
    DirBuilder::new().recursive(true).create("certificates")?;

    write("certificates/ca.crt", creator.ca)?;
    write("certificates/client.crt", creator.certificate)?;
    write("certificates/client.key", creator.private_key)?;
    write("certificates/uuid", creator.network.id.to_string())?;

    Ok(())
}

///Load certifcates and return a creator object
pub fn load_certs() -> Result<Creator, Box<dyn Error>> {
    let ca = read_to_string("certificates/ca.crt")?;
    let certificate = read_to_string("certificates/client.crt")?;
    let private_key = read_to_string("certificates/client.key")?;
    let id = read_to_string("certificates/uuid")?;

    Ok(Creator {
        ca,
        certificate,
        private_key,
        network: CreatorNetwork {
            id: Uuid::parse_str(&id)?,
        },
    })
}
