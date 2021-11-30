use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use uuid::Uuid;

pub struct RequestBuilder<'a> {
    username: &'a str,
    password: &'a str,
    server: WappstoServers,
}

impl<'a> RequestBuilder<'a> {
    pub fn new() -> Self {
        Self {
            username: "",
            password: "",
            server: WappstoServers::PROD,
        }
    }

    pub fn with_credentials(mut self, username: &'a str, password: &'a str) -> Self {
        self.username = username;
        self.password = password;
        self
    }

    pub fn to_server(mut self, server: WappstoServers) -> Self {
        self.server = server;
        self
    }

    pub fn send(self) -> Result<Creator, Box<dyn Error>> {
        let client = Client::new();
        let base_url = match self.server {
            WappstoServers::PROD => "https://wappsto.com/services/",
            WappstoServers::QA => "https://qa.wappsto.com/services/",
        };
        let credentials = json!({
            "username": self.username,
            "password": self.password
        });
        let session_response: Session = client
            .post(base_url.to_owned() + "2.0/session")
            .json(&credentials)
            .send()?
            .json()?;
        let creator: Creator = client
            .post(base_url.to_owned() + "2.1/creator")
            .header("x-session", session_response.meta.id.to_string())
            .json(&json!({}))
            .send()?
            .json()?;

        Ok(creator)
    }
}

impl<'a> Default for RequestBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct WappstoHttpError;

pub enum WappstoServers {
    PROD,
    QA,
}

#[derive(Deserialize)]
pub struct Creator {
    pub ca: String,
    pub certificate: String,
    pub private_key: String,
    pub network: CreatorNetwork,
}

#[derive(Deserialize)]
pub struct CreatorNetwork {
    pub id: Uuid,
}

#[derive(Deserialize)]
struct Session {
    meta: SessionMeta,
}

#[derive(Deserialize)]
struct SessionMeta {
    pub id: Uuid,
}
