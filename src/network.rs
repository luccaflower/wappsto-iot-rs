use std::{error::Error, fmt::Display};

use uuid::Uuid;

use crate::{
    connection::{Connectable, Connection},
    fs_store::{FsStore, Store},
};

pub struct Network<'a, C = Connection, S = FsStore>
where
    C: Connectable + Default,
    S: Store + Default,
{
    pub name: &'a str,
    pub id: Uuid,
    connection: C,
    #[allow(dead_code)]
    store: S,
}

impl<'a> Network<'a, Connection> {}

impl<'a, C, S> Network<'a, C, S>
where
    C: Connectable + Default,
    S: Store + Default,
{
    pub fn new(name: &'a str) -> Result<Self, Box<dyn Error>> {
        let store = S::default();
        let id = store.load_certs()?.id;
        Ok(Self {
            name,
            id,
            connection: C::default(),
            store,
        })
    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.connection.start()
    }

    #[cfg(test)]
    pub fn connection(&self) -> &C {
        &self.connection
    }

    #[cfg(test)]
    pub fn store(&self) -> &S {
        &self.store
    }
}

#[derive(Debug)]
struct DummyError;

impl Display for DummyError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Error for DummyError {}
