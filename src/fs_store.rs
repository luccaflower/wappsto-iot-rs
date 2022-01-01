use serde_json;
use std::error::Error;
use std::fs::{read_to_string, write, DirBuilder, File};
use uuid::Uuid;

use crate::certs::Certs;
use crate::create_network::{Creator, CreatorNetwork};
use crate::schema::Schema;

pub struct FsStore {
    certificates: String,
}
pub trait Store {
    fn load_certs(&self) -> Result<Certs, Box<dyn Error>>;
    fn save_schema(&mut self, schema: Schema) -> Result<(), Box<dyn Error>>;
}

impl FsStore {
    pub fn new(certificates: &str) -> Self {
        Self {
            certificates: String::from(certificates),
        }
    }

    pub fn save_certs_self(&self, creator: Creator) -> Result<(), Box<dyn Error>> {
        DirBuilder::new().recursive(true).create("certificates")?;

        write(self.certificates.clone() + "ca.crt", &creator.ca)?;
        write(
            self.certificates.clone() + "client.crt",
            &creator.certificate,
        )?;
        write(
            self.certificates.clone() + "client.key",
            &creator.private_key,
        )?;

        Ok(())
    }
}

impl Store for FsStore {
    fn load_certs(&self) -> Result<Certs, Box<dyn Error>> {
        unimplemented!("Load certs not implemented for FsStore")
    }

    fn save_schema(&mut self, _schema: Schema) -> Result<(), Box<dyn Error>> {
        unimplemented!("Save schema not implemented for FsStore")
    }
}
impl Default for FsStore {
    fn default() -> Self {
        Self::new("certifcates/")
    }
}

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
