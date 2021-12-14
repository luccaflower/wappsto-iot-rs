use openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslStream, SslVerifyMode};

use std::error::Error;
use std::net::TcpStream;
use std::path::Path;

pub struct Connection;
pub trait Connectable {
    fn start(&mut self) -> Result<(), Box<dyn Error>>;
}

impl Connectable for Connection {
    fn start(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl Connection {
    pub fn new() -> Self {
        Connection {}
    }
}

impl Default for Connection {
    fn default() -> Self {
        Self::new()
    }
}

pub fn start() -> Result<SslStream<TcpStream>, Box<dyn Error>> {
    let mut ctx = SslConnector::builder(SslMethod::tls())?;

    ctx.set_ca_file(Path::new("certificates/ca.crt"))?;
    ctx.set_certificate_file(Path::new("certificates/client.crt"), SslFiletype::PEM)?;
    ctx.set_private_key_file(Path::new("certificates/client.key"), SslFiletype::PEM)?;
    ctx.set_verify(SslVerifyMode::NONE);

    let stream = TcpStream::connect("qa.wappsto.com:53005")?;
    let stream = ctx.build().connect("qa.wappsto.com", stream)?;
    Ok(stream)
}
