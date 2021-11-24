use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Schema {
    pub meta: Meta,
    pub device: Vec<Device>,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub id: Uuid,
}

impl Schema {
    pub fn new(id: Uuid) -> Schema {
        Schema {
            meta: Meta { id },
            device: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Device;

pub struct SchemaBuilder {
    id: Uuid,
    device: Vec<Device>,
}

impl SchemaBuilder {
    pub fn new(id: Uuid) -> SchemaBuilder {
        SchemaBuilder { id, device: vec![] }
    }

    pub fn create(self) -> Schema {
        Schema {
            meta: Meta { id: self.id },
            device: self.device,
        }
    }
}
