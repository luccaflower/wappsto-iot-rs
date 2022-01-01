use async_trait::async_trait;
use openssl::{
    ssl::{SslConnector, SslFiletype, SslMethod, SslStream, SslVerifyMode},
    x509::store::X509StoreBuilder,
};

use std::{error::Error, path::Path};
use tokio::{
    io::{split, ReadHalf, WriteHalf},
    net::TcpStream,
};

use crate::{certs::Certs, rpc::Rpc};

pub struct Connection {
    certs: Certs,
    read: Option<ReadHalf<TcpStream>>,
    write: Option<WriteHalf<TcpStream>>,
}

#[async_trait]
pub trait Connect {
    fn new(certs: Certs) -> Self;
    async fn start(&mut self) -> Result<(), Box<dyn Error>>;
    fn stop(&mut self);
    fn send(&mut self, rpc: Rpc);
}

#[async_trait]
impl Connect for Connection {
    fn new(certs: Certs) -> Self {
        Self {
            certs,
            read: None,
            write: None,
        }
    }
    async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let mut store = X509StoreBuilder::new()?;
        store.add_cert(self.certs.ca.clone())?;
        store.add_cert(self.certs.certificate.clone())?;
        let store = store.build();
        let mut ctx = SslConnector::builder(SslMethod::tls())?;
        ctx.set_cert_store(store);
        ctx.set_private_key(&self.certs.private_key)?;
        let stream = TcpStream::connect("qa.wappsto.com:53005").await?;
        let (read, write) = split(stream);
        self.read = Some(read);
        self.write = Some(write);

        Ok(())
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

    //let stream = TcpStream::connect("qa.wappsto.com:53005")?;
    //let stream = ctx.build().connect("qa.wappsto.com", stream)?;
    //Ok(stream)
    unimplemented!()
}
