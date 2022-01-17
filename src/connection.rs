use openssl::ssl::{SslConnector, SslMethod};

use std::{error::Error, net::TcpStream, sync::mpsc::Sender};

use crate::{
    certs::Certs,
    communication::{self, CallbackMap},
};

const DEV: &[&str] = &["dev.", ":52005"];
const QA: &[&str] = &["qa.", ":53005"];
const STAGING: &[&str] = &["staging.", ":54005"];
const PROD: &[&str] = &["", ":443"];
const BASE_URL: &str = "wappsto.com";

pub struct Connection {
    certs: Certs,
    #[allow(dead_code)]
    url: &'static [&'static str],
}

pub trait Connect<Se>
where
    Se: WrappedSend,
{
    fn new(certs: Certs, server: WappstoServers) -> Self;
    fn start(&self, callbacks: CallbackMap) -> Result<Se, Box<dyn Error>>;
}

impl Connect<SendChannel> for Connection {
    fn new(certs: Certs, server: WappstoServers) -> Self {
        let url = match server {
            WappstoServers::DEV => DEV,
            WappstoServers::QA => QA,
            WappstoServers::STAGING => STAGING,
            WappstoServers::PROD => PROD,
        };
        Self { certs, url }
    }

    fn start(&self, callbacks: CallbackMap) -> Result<SendChannel, Box<dyn Error>> {
        let mut ctx = SslConnector::builder(SslMethod::tls())?;
        ctx.cert_store_mut().add_cert(self.certs.ca.clone())?;
        ctx.set_certificate(&self.certs.certificate)?;
        ctx.set_private_key(&self.certs.private_key)?;

        let stream = TcpStream::connect(&(String::from(self.url[0]) + BASE_URL + self.url[1]))?;
        let stream = ctx
            .build()
            .connect(&(String::from(self.url[0]) + BASE_URL), stream)?;

        stream.get_ref().set_nonblocking(true)?;

        Ok(SendChannel::new(communication::start(callbacks, stream)))
    }
}

pub struct SendChannel {
    send: Sender<String>,
}

pub trait WrappedSend {
    fn send(&self, msg: String) -> Result<(), Box<dyn Error>>;
}

impl WrappedSend for SendChannel {
    fn send(&self, msg: String) -> Result<(), Box<dyn Error>> {
        self.send.send(msg)?;
        Ok(())
    }
}

impl SendChannel {
    pub fn new(send: Sender<String>) -> Self {
        Self { send }
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
