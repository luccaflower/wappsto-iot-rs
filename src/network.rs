use std::{error::Error, fmt::Display};

use uuid::Uuid;

use crate::{
    connection::{Connectable, Connection},
    fs_store::{FsStore, Store},
};

pub struct Network<'a, C = Connection<'a>, S = FsStore>
where
    C: Connectable<'a>,
    S: Store<'a> + Default,
{
    pub name: &'a str,
    pub id: Uuid,
    connection: C,
    #[allow(dead_code)]
    store: S,
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
