#[allow(dead_code)]
pub mod rest {
    use std::{env, error::Error};
    use uuid::Uuid;
    use wappsto_iot_rs::{certs::Certs, create_network::RequestBuilder, fs_store::FsStore};

    const DEV: &str = "dev.";
    const QA: &str = "qa.";
    const STAGING: &str = "staging.";
    const PROD: &str = "";
    pub struct RestSession {
        url: String,
        session: Uuid,
    }

    impl RestSession {
        pub fn new(_username: &str, _password: &str, _server: RestServer) -> Self {
            todo!()
        }

        pub fn control(&self, _id: Uuid) -> Result<(), Box<dyn Error>> {
            todo!()
        }
    }

    pub enum RestServer {
        Dev,
        Qa,
        Staging,
        Prod,
    }

    pub fn create_network() -> Result<(), Box<dyn Error>> {
        dotenv::dotenv().ok();
        let (username, password) = credentials();
        let creator = RequestBuilder::new()
            .with_credentials(&username, &password)
            .to_server(wappsto_iot_rs::create_network::WappstoServers::QA)
            .send()?;
        let certs = Certs::new(&creator.ca, &creator.certificate, &creator.private_key).unwrap();
        FsStore::default().save_certs(certs).unwrap();
        Ok(())
    }

    pub fn credentials() -> (String, String) {
        dotenv::dotenv().ok();
        let username = env::var("WAPPSTO_USERNAME").unwrap();
        let password = env::var("WAPPSTO_PASSWORD").unwrap();
        (username, password)
    }
}
