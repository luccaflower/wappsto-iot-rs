use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Schema {
    pub id: Uuid,
    pub device: Vec<Device>,
}

impl Schema {
    pub fn new(id: Uuid) -> Schema {
        Schema { id, device: vec![] }
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
            id: self.id,
            device: self.device,
        }
    }
}
