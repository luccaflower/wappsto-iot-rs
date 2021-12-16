use openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslStream, SslVerifyMode};

use std::error::Error;
use std::net::TcpStream;
use std::path::Path;

use crate::certs::Certs;

pub struct Connection<'a> {
    #[allow(dead_code)]
    certs: Certs<'a>,
}
pub trait Connectable<'a> {
    fn new(certs: Certs<'a>) -> Self;
    fn start(&mut self) -> Result<(), Box<dyn Error>>;
    fn stop(&mut self);
}

impl<'a> Connectable<'a> for Connection<'a> {
    fn new(certs: Certs<'a>) -> Self {
        Self { certs }
    }
    fn start(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn stop(&mut self) {}
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
