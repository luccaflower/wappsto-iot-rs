#[allow(dead_code)]
pub mod rest {
    use reqwest::{
        blocking::{Client, ClientBuilder},
        header::{HeaderMap, HeaderValue},
    };
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::{env, error::Error, io::Read};
    use uuid::Uuid;
    use wappsto_iot_rs::{certs::Certs, create_network::RequestBuilder, fs_store::FsStore};

    const DEV: &str = "https://dev.";
    const QA: &str = "https://qa.";
    const STAGING: &str = "https://staging.";
    const PROD: &str = "https://";
    const BASE_URL: &str = "wappsto.com/services/";
    const VERSION_2: &str = "2.0/";
    const VERSION_2_1: &str = "2.1/";

    pub struct RestSession {
        url: String,
        client: Client,
    }

    impl RestSession {
        pub fn new(username: &str, password: &str, server: RestServer) -> Self {
            let url = String::from(match server {
                RestServer::Dev => DEV,
                RestServer::Qa => QA,
                RestServer::Staging => STAGING,
                RestServer::Prod => PROD,
            }) + BASE_URL;
            let credentials = Credentials::new(username, password);

            let session_response: Session = Client::new()
                .post(url.clone() + VERSION_2 + "session")
                .json(&json!(&credentials))
                .send()
                .expect("Failed to log in")
                .json()
                .expect("Failed to deserialize");
            let id = session_response.meta.id.to_string();

            let mut headers = HeaderMap::new();
            headers.insert("X-Session", HeaderValue::from_str(&id).unwrap());
            let client = ClientBuilder::new()
                .default_headers(headers)
                .build()
                .expect("Failed to create client");
            Self { url, client }
        }

        pub fn control(&self, id: Uuid, data: &str) -> Result<(), Box<dyn Error>> {
            self.client
                .post(self.url.clone() + VERSION_2 + "state/" + &id.to_string())
                .json(&json!({ "data": data }))
                .send()
                .unwrap();
            Ok(())
        }
    }

    pub enum RestServer {
        Dev,
        Qa,
        Staging,
        Prod,
    }

    #[derive(Serialize)]
    struct Credentials {
        username: String,
        password: String,
    }

    #[derive(Deserialize)]
    struct Session {
        pub meta: SessionMeta,
    }

    #[derive(Deserialize)]
    struct SessionMeta {
        pub id: Uuid,
    }

    impl Credentials {
        pub fn new(username: &str, password: &str) -> Self {
            Self {
                username: String::from(username),
                password: String::from(password),
            }
        }
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
