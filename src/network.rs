use std::{error::Error, fmt::Display};

pub struct Network;

impl Network {
    pub fn builder(_name: &str) -> NetworkBuilder {
        NetworkBuilder
    }
}

pub struct NetworkBuilder;

impl NetworkBuilder {
    pub fn create(self) -> Result<Network, Box<dyn Error>> {
        Err(Box::new(DummyError {}))
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
