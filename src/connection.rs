use async_trait::async_trait;

use tokio_rustls::{
    client::TlsStream,
    rustls::{Certificate, ClientConfig, PrivateKey, RootCertStore, ServerName},
    TlsConnector,
};

use std::{error::Error, sync::Arc};
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

use crate::{certs::Certs, rpc::Rpc};

const DEV: &str = "dev.wappsto.com:52005";
const QA: &str = "qa.wappsto.com:53005";
const STAGING: &str = "staging.wappsto.com:54005";
const PROD: &str = "prod.wappsto.com:443";

pub struct Connection {
    certs: Certs,
    read: Option<ReadHalf<TlsStream<TcpStream>>>,
    write: Option<WriteHalf<TlsStream<TcpStream>>>,
    url: &'static str,
}

#[async_trait]
pub trait Connect {
    fn new(certs: Certs, server: WappstoServers) -> Self;
    async fn start(&mut self) -> Result<(), Box<dyn Error>>;
    async fn send(&mut self, rpc: Rpc);
    fn stop(&mut self);
}

#[async_trait]
impl Connect for Connection {
    fn new(certs: Certs, server: WappstoServers) -> Self {
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
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store
            .add(&Certificate(self.certs.ca.to_der().unwrap()))
            .expect("adding root certificate");
        println!("adding root certificate");

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_single_cert(
                vec![Certificate(self.certs.certificate.to_pem().unwrap())],
                PrivateKey(self.certs.private_key.private_key_to_der().unwrap()),
            )
            .expect("adding client certificate");
        println!("adding client certificate");
        let config = TlsConnector::from(Arc::new(config));
        println!("connecting...");
        let stream = TcpStream::connect(self.url).await?;
        let stream = config
            .connect(ServerName::try_from("qa.wappsto.com").unwrap(), stream)
            .await?;
        println!("connected");
        let (read, write) = split(stream);
        self.read = Some(read);
        self.write = Some(write);

        Ok(())
    }

    async fn send(&mut self, rpc: Rpc) {
        println!("{}", &serde_json::to_string(&rpc).unwrap());
        self.write
            .as_mut()
            .unwrap()
            .write_all(&serde_json::to_vec(&rpc).unwrap())
            .await
            .unwrap();
        let mut buf = [0u8; 1024];
        self.read
            .as_mut()
            .unwrap()
            .read(&mut buf[..])
            .await
            .unwrap();
        println!("{}", buf.iter().map(|c| *c as char).collect::<String>());
    }

    fn stop(&mut self) {
        todo!("stop connection")
    }
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
