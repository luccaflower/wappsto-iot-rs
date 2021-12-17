use std::{collections::HashMap, error::Error};

use uuid::Uuid;

use crate::{
    connection::{Connectable, Connection},
    fs_store::{FsStore, Store},
    schema::{DeviceSchema, Schema},
};

pub struct Network<'a, C = Connection<'a>, S = FsStore>
where
    C: Connectable<'a>,
    S: Store<'a> + Default,
{
    pub name: &'a str,
    pub id: Uuid,
    connection: C,
    store: S,
    devices: HashMap<&'a str, Device<'a>>,
}

impl<'a, C, S> Network<'a, C, S>
where
    C: Connectable<'a>,
    S: Store<'a> + Default,
{
    pub fn new(name: &'a str) -> Result<Self, Box<dyn Error>> {
        let store = S::default();
        let certs = store.load_certs()?;
        Ok(Self {
            name,
            id: certs.id,
            connection: C::new(certs),
            store,
            devices: HashMap::new(),
        })
    }

    pub fn create_device(&mut self, name: &'a str) -> &'a Device {
        self.devices.entry(name).or_default()
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
        let mut schema = Schema::new(self.name, self.id);
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
    pub fn devices(&mut self) -> &mut HashMap<&'a str, Device<'a>> {
        &mut self.devices
    }
}

pub struct Device<'a> {
    pub id: Uuid,
    values: HashMap<&'a str, Value<'a>>,
}

impl<'a> Device<'a> {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            values: HashMap::new(),
        }
    }

    //TODO: satisfy the linter, probably
    #[allow(clippy::mut_from_ref)]
    pub fn create_value(
        &mut self,
        name: &'a str,
        permission: ValuePermission<'a>,
    ) -> &'a mut Value {
        self.values
            .entry(name)
            .or_insert_with(|| Value::new(permission))
    }
}

impl Default for Device<'_> {
    fn default() -> Self {
        Self::new(Uuid::new_v4())
    }
}

pub struct Value<'a> {
    #[allow(dead_code)]
    control: Option<&'a mut dyn FnMut(String)>,
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

    #[cfg(test)]
    pub fn control(&mut self, data: String) {
        self.control.as_mut().unwrap()(data)
    }
}

pub enum ValuePermission<'a> {
    RW(&'a mut dyn FnMut(String)),
    R,
    W(&'a mut dyn FnMut(String)),
}
