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

    pub fn send(self) -> Result<Uuid, WappstoHttpError> {
        Err(WappstoHttpError)
    }
}

impl<'a> Default for RequestBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WappstoHttpError;

pub enum WappstoServers {
    PROD,
    QA,
}
