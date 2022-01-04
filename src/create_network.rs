use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::marker::PhantomData;
use uuid::Uuid;

#[doc(hidden)]
pub struct NoCredentials;

#[doc(hidden)]
pub struct WithCredentials;

#[doc(hidden)]
pub trait Credentials {}

impl Credentials for NoCredentials {}
impl Credentials for WithCredentials {}

///Make a request to the Wappsto REST API in order to create a new network and retrieve its UUID
///and SSL certificates.
///# Example
///```no_run
/// # use wappsto_iot_rs::create_network::*;
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
///     let creator = RequestBuilder::new()
///         .with_credentials("username", "password")
///         .to_server(WappstoServers::PROD)
///         .send()?;
/// #   Ok(())
/// # }
///```
pub struct RequestBuilder<'a, C: Credentials> {
    username: &'a str,
    password: &'a str,
    server: WappstoServers,
    credentials_state: PhantomData<C>,
}

impl<'a> RequestBuilder<'a, NoCredentials> {
    pub fn new() -> Self {
        RequestBuilder {
            username: "",
            password: "",
            server: WappstoServers::PROD,
            credentials_state: PhantomData,
        }
    }
}

impl<'a, C> RequestBuilder<'a, C>
where
    C: Credentials,
{
    pub fn with_credentials(
        self,
        username: &'a str,
        password: &'a str,
    ) -> RequestBuilder<'a, WithCredentials> {
        RequestBuilder {
            username,
            password,
            server: self.server,
            credentials_state: PhantomData,
        }
    }

    pub fn to_server(mut self, server: WappstoServers) -> Self {
        self.server = server;
        self
    }
}

impl<'a> RequestBuilder<'a, WithCredentials> {
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
        let response = client
            .post(base_url.to_owned() + "2.1/creator")
            .header("x-session", session_response.meta.id.to_string())
            .json(&json!({
                "manufacturer_as_owner": true
            }))
            .send()?;

        let creator: Creator = response.json()?;
        println!("Network ID from creator:  {}", creator.network.id);

        Ok(creator)
    }
}

impl<'a> Default for RequestBuilder<'a, NoCredentials> {
    fn default() -> Self {
        Self::new()
    }
}

///The servers you can make requests to. Defaults to PROD.
pub enum WappstoServers {
    PROD,
    QA,
}

///The creator object contains the required SSL certificates and network UUID
#[derive(Deserialize)]
pub struct Creator {
    pub ca: String,
    pub certificate: String,
    pub private_key: String,
    pub network: CreatorNetwork,
}

#[doc(hidden)]
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
