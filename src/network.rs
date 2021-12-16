use std::{collections::HashMap, error::Error};

use uuid::Uuid;

use crate::{
    connection::{Connectable, Connection},
    fs_store::{FsStore, Store},
    schema::{Schema, SchemaBuilder},
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
    devices: HashMap<&'a str, Device>,
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

    pub fn create_device(&mut self, name: &'a str) -> &Device {
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
        SchemaBuilder::new(self.id).create()
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
    pub fn devices(&mut self) -> &mut HashMap<&'a str, Device> {
        &mut self.devices
    }
}

pub struct Device {
    pub id: Uuid,
}

impl Device {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new()
    }
}
