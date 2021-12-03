use crate::create_network::Creator;
use std::error::Error;
use std::io;
use std::io::{Read, Write};

pub fn start<S>(_creator: Creator) -> Result<S, Box<dyn Error>>
where
    S: Read + Write,
{
    Err(Box::new(io::Error::new(io::ErrorKind::Other, "")))
}
