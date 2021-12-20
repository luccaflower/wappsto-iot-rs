use std::{collections::HashMap, error::Error};

use uuid::Uuid;

use crate::{
    connection::{Connectable, Connection},
    fs_store::{FsStore, Store},
    schema::{DeviceSchema, Schema},
};

pub struct Network<'a, C = Connection, S = FsStore>
where
    C: Connectable,
    S: Store + Default,
{
    pub name: String,
    pub id: Uuid,
    connection: C,
    store: S,
    devices: HashMap<String, Device<'a>>,
}

impl<'a, C, S> Network<'a, C, S>
where
    C: Connectable,
    S: Store + Default,
{
    pub fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        let store = S::default();
        let certs = store.load_certs()?;
        Ok(Self {
            name: String::from(name),
            id: certs.id,
            connection: C::new(certs),
            store,
            devices: HashMap::new(),
        })
    }

    pub fn create_device(&mut self, name: &str) -> &mut Device<'a> {
        self.devices.entry(String::from(name)).or_default()
    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.connection.start()
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        self.connection.stop();
        self.store.save_schema(self.schema())?;
        Ok(())
    }

    fn schema(&self) -> Schema {
        let mut schema = Schema::new(&self.name, self.id);
        schema.device = self
            .devices
            .iter()
            .map(|(name, device)| DeviceSchema::new(name, device.id))
            .collect();
        schema
    }

    #[cfg(test)]
    pub fn connection(&self) -> &C {
        &self.connection
    }

    #[cfg(test)]
    pub fn store(&self) -> &S {
        &self.store
    }

    #[cfg(test)]
    pub fn devices(&mut self) -> &mut HashMap<String, Device<'a>> {
        &mut self.devices
    }
}

pub struct Device<'a> {
    pub id: Uuid,
    values: HashMap<String, Value<'a>>,
}

impl<'a> Device<'a> {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            values: HashMap::new(),
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn create_value(&mut self, name: &str, permission: ValuePermission<'a>) -> &mut Value<'a> {
        self.values
            .entry(String::from(name))
            .or_insert_with(|| Value::new(permission))
    }

    #[cfg(test)]
    pub fn values(&self) -> &HashMap<String, Value<'a>> {
        &self.values
    }
}

impl Default for Device<'_> {
    fn default() -> Self {
        Self::new(Uuid::new_v4())
    }
}

pub struct Value<'a> {
    #[allow(dead_code)]
    control: Option<Box<dyn FnMut(String) + 'a>>,
}

impl<'a> Value<'a> {
    pub fn new(permission: ValuePermission<'a>) -> Self {
        Self {
            control: match permission {
                ValuePermission::RW(f) | ValuePermission::W(f) => Some(f),
                ValuePermission::R => None,
            },
        }
    }
}

impl Value<'_> {
    #[cfg(test)]
    pub fn control(&mut self, data: String) {
        self.control.as_mut().unwrap()(data)
    }
}

pub enum ValuePermission<'a> {
    RW(Box<dyn FnMut(String) + 'a>),
    R,
    W(Box<dyn FnMut(String) + 'a>),
}
