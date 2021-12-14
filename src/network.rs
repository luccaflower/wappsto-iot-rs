use std::{error::Error, fmt::Display};

use crate::connection::{Connectable, Connection};

pub struct Network<C>
where
    C: Connectable,
{
    pub connection: C,
}

impl Network<Connection> {
    pub fn new(_name: &str) -> Self {
        Self {
            connection: Connection::new(),
        }
    }
}

impl<C> Network<C>
where
    C: Connectable,
{
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.connection.start()
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
