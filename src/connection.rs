use openssl::ssl::{SslConnector, SslFiletype, SslMethod};

use std::{
    collections::HashMap,
    error::Error,
    path::Path,
    sync::{mpsc::Sender, Arc},
    thread::sleep,
    time::Duration,
};

use crate::{certs::Certs, communication};

const DEV: &[&str] = &["dev.", ":52005"];
const QA: &[&str] = &["qa.", ":53005"];
const STAGING: &[&str] = &["staging.", ":54005"];
const PROD: &[&str] = &["", ":443"];
#[allow(dead_code)]
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
    fn start(&mut self) -> Result<Box<Se>, Box<dyn Error>>;
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

    fn start(&mut self) -> Result<Box<SendChannel>, Box<dyn Error>> {
        sleep(Duration::from_millis(1000));
        let mut ctx = SslConnector::builder(SslMethod::tls())?;
        println!("set ca");
        ctx.set_ca_file(Path::new("certificates/ca.crt"))?;
        println!("ca: {:?}", &self.certs.ca);
        println!("set cert");
        ctx.set_certificate_file(Path::new("certificates/client.crt"), SslFiletype::PEM)?;
        println!("set private key");
        ctx.set_private_key_file(Path::new("certificates/client.key"), SslFiletype::PEM)?;

        println!("raw socket");
        let stream = std::net::TcpStream::connect("qa.wappsto.com:53005")?;
        println!("tls wrapped socket");
        let stream = ctx.build().connect("qa.wappsto.com", stream)?;

        println!("set non-blocking");
        stream.get_ref().set_nonblocking(true)?;

        Ok(Box::new(SendChannel::new(communication::start(
            HashMap::new(),
            stream,
        ))))
    }
}

pub struct SendChannel {
    send: Arc<Sender<String>>,
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
    pub fn new(send: Arc<Sender<String>>) -> Self {
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
