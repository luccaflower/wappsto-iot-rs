use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
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
            name: "".to_owned(),
            meta: Meta { id },
            device: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Device;

impl Device {
    pub fn new() -> Self {
        Device {}
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SchemaBuilder {
    name: String,
    id: Uuid,
    device: Vec<Device>,
}

impl SchemaBuilder {
    pub fn new(id: Uuid) -> Self {
        SchemaBuilder {
            name: "".to_owned(),
            id,
            device: vec![],
        }
    }

    pub fn named(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn add_device(mut self, device: Device) -> Self {
        self.device.push(device);
        self
    }

    pub fn create(self) -> Schema {
        Schema {
            name: self.name,
            meta: Meta { id: self.id },
            device: self.device,
        }
    }
}
