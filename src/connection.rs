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

const DEV: &str = "dev.wappsto.com:52005";
const QA: &str = "qa.wappsto.com:53005";
const STAGING: &str = "staging.wappsto.com:54005";
const PROD: &str = "prod.wappsto.com:443";

pub struct Connection {
    certs: Certs,
    read: Option<ReadHalf<TcpStream>>,
    write: Option<WriteHalf<TcpStream>>,
    url: &'static str,
}

#[async_trait]
pub trait Connect {
    fn new_servers(certs: Certs, server: WappstoServers) -> Self;
    async fn start(&mut self) -> Result<(), Box<dyn Error>>;
    fn stop(&mut self);
    fn send(&mut self, rpc: Rpc);
}

#[async_trait]
impl Connect for Connection {
    fn new_servers(certs: Certs, server: WappstoServers) -> Self {
        let url = match server {
            WappstoServers::DEV => DEV,
            WappstoServers::QA => QA,
            WappstoServers::STAGING => STAGING,
            WappstoServers::PROD => PROD,
        };
        Self {
            certs,
            read: None,
            write: None,
            url,
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
        let stream = TcpStream::connect(self.url).await?;
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

pub enum WappstoServers {
    DEV,
    QA,
    STAGING,
    PROD,
}

impl Default for WappstoServers {
    fn default() -> Self {
        Self::PROD
    }
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
