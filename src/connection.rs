use openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslStream, SslVerifyMode};

use std::error::Error;
use std::net::TcpStream;
use std::path::Path;

use crate::{certs::Certs, rpc::Rpc};

pub struct Connection {
    #[allow(dead_code)]
    certs: Certs,
}
pub trait Connect {
    fn new(certs: Certs) -> Self;
    fn start(&mut self) -> Result<(), Box<dyn Error>>;
    fn stop(&mut self);
    fn send(&mut self, rpc: Rpc);
}

impl Connect for Connection {
    fn new(certs: Certs) -> Self {
        Self { certs }
    }
    fn start(&mut self) -> Result<(), Box<dyn Error>> {
        todo!("start connection")
    }
    fn stop(&mut self) {
        todo!("stop connection")
    }

    fn send(&mut self, _rpc: Rpc) {}
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
