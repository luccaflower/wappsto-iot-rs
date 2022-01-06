use async_trait::async_trait;

use rustls::OwnedTrustAnchor;
use tokio_rustls::{
    client::TlsStream,
    rustls::{Certificate, ClientConfig, PrivateKey, RootCertStore, ServerName},
    TlsConnector,
};
use webpki_roots::TLS_SERVER_ROOTS;

use std::{error::Error, sync::Arc, thread::sleep, time::Duration};
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

use crate::{certs::Certs, rpc::Rpc};

const DEV: &[&str] = &["dev.", ":52005"];
const QA: &[&str] = &["qa.", ":53005"];
const STAGING: &[&str] = &["staging.", ":54005"];
const PROD: &[&str] = &["", ":443"];
const BASE_URL: &str = "wappsto.com";

pub struct Connection {
    certs: Certs,
    read: Option<ReadHalf<TlsStream<TcpStream>>>,
    write: Option<WriteHalf<TlsStream<TcpStream>>>,
    url: &'static [&'static str],
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
        sleep(Duration::from_millis(1000));
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.add_server_trust_anchors(TLS_SERVER_ROOTS.0.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));
        root_cert_store
            .add(&Certificate(self.certs.ca.to_der().unwrap()))
            .unwrap();

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_single_cert(
                vec![Certificate(self.certs.certificate.to_der().unwrap())],
                PrivateKey(self.certs.private_key.private_key_to_der().unwrap()),
            )
            .expect("adding client certificate");
        let connector = TlsConnector::from(Arc::new(config));
        let stream = TcpStream::connect(self.url[0].to_owned() + BASE_URL + self.url[1]).await?;
        let stream = connector
            .connect(
                ServerName::try_from((self.url[0].to_owned() + BASE_URL).as_str()).unwrap(),
                stream,
            )
            .await?;
        let (read, write) = split(stream);
        self.read = Some(read);
        self.write = Some(write);

        Ok(())
    }

    async fn send(&mut self, rpc: Rpc) {
        self.write
            .as_mut()
            .unwrap()
            .write_all(serde_json::to_string(&rpc).unwrap().as_bytes())
            .await
            .unwrap();
        let mut buf = [0; 4096];
        sleep(Duration::from_millis(1000));
        let bytes = self.read.as_mut().unwrap().read(&mut buf).await.unwrap();
        println!(
            "{:?}",
            buf[..bytes].iter().map(|x| *x as char).collect::<String>()
        );
    }

    fn stop(&mut self) {
        self.write = None;
        self.read = None;
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
