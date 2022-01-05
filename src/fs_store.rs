use serde_json;
use std::error::Error;
use std::fs::{read_to_string, write, DirBuilder, File};
use uuid::Uuid;

use crate::certs::Certs;
use crate::schema::Schema;

const CA_FILE: &str = "ca.crt";
const CERT_FILE: &str = "client.crt";
const KEY_FILE: &str = "client.key";

pub struct FsStore {
    certificates: String,
    network_schema: String,
}
pub trait Store {
    fn load_certs(&self) -> Result<Certs, Box<dyn Error>>;
    fn save_schema(&mut self, schema: Schema) -> Result<(), Box<dyn Error>>;
    fn load_schema(&self, id: Uuid) -> Option<Schema>;
}

impl FsStore {
    pub fn new(certificates: &str, network_schema: &str) -> Self {
        Self {
            certificates: String::from(certificates),
            network_schema: String::from(network_schema),
        }
    }

    pub fn save_certs(&self, certs: Certs) -> Result<(), Box<dyn Error>> {
        DirBuilder::new()
            .recursive(true)
            .create(&self.certificates)?;

        write(
            self.certificates.clone() + CA_FILE,
            &certs.ca.to_pem().unwrap(),
        )?;
        write(
            self.certificates.clone() + CERT_FILE,
            &certs.certificate.to_pem().unwrap(),
        )?;
        write(
            self.certificates.clone() + KEY_FILE,
            &certs.private_key.private_key_to_pem_pkcs8().unwrap(),
        )?;

        Ok(())
    }
}

impl Store for FsStore {
    ///Load certifcates from file store
    fn load_certs(&self) -> Result<Certs, Box<dyn Error>> {
        let ca = read_to_string(self.certificates.clone() + CA_FILE)?;
        let certificate = read_to_string(self.certificates.clone() + CERT_FILE)?;
        let private_key = read_to_string(self.certificates.clone() + KEY_FILE)?;

        Certs::new(&ca, &certificate, &private_key)
    }

    ///Save network schema to data store
    fn save_schema(&mut self, schema: Schema) -> Result<(), Box<dyn Error>> {
        DirBuilder::new()
            .recursive(true)
            .create(&self.network_schema)
            .unwrap();

        Ok(serde_json::to_writer(
            &File::create(self.network_schema.clone() + &schema.meta.id.to_string() + ".json")
                .unwrap(),
            &schema,
        )?)
    }
    ///Load network schema from data store
    fn load_schema(&self, id: Uuid) -> Option<Schema> {
        let contents =
            match read_to_string(String::from(&self.network_schema) + &id.to_string() + ".json") {
                Ok(s) => s,
                Err(_) => return None,
            };
        match serde_json::from_str(&contents) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }
}
impl Default for FsStore {
    fn default() -> Self {
        Self::new("certificates/", "network_instance/")
    }
}
