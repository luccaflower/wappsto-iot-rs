use serde::Serialize;
use uuid::Uuid;

use crate::schema::Schema;

#[derive(Serialize)]
pub struct Rpc {
    jsonrpc: String,
    method: RpcMethod,
    id: String,
    params: RpcParams,
}

impl Rpc {
    pub fn builder() -> RpcBuilder {
        RpcBuilder::new()
    }

    pub fn new(method: RpcMethod, rpc_type: RpcType, data: Schema) -> Rpc {
        Self {
            jsonrpc: String::from("2.0"),
            method,
            id: Uuid::new_v4().to_string(),
            params: RpcParams::new(rpc_type, data),
        }
    }
}

pub struct RpcBuilder {
    method: RpcMethod,
    rpc_type: RpcType,
    data: Schema,
}

impl RpcBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            method: RpcMethod::Post,
            rpc_type: RpcType::Network,
            data: Schema::new("", Uuid::new_v4()),
        }
    }

    pub fn method(mut self, method: RpcMethod) -> Self {
        self.method = method;
        self
    }

    pub fn on_type(mut self, rpc_type: RpcType) -> Self {
        self.rpc_type = rpc_type;
        self
    }

    pub fn data(mut self, schema: Schema) -> Self {
        self.data = schema;
        self
    }

    pub fn create(self) -> Rpc {
        Rpc::new(self.method, self.rpc_type, self.data)
    }
}

#[derive(Serialize)]
pub enum RpcMethod {
    Post,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RpcType {
    Network,
}

#[derive(Serialize)]
pub struct RpcParams {
    url: String,
    data: Schema,
}

impl RpcParams {
    pub fn new(rpc_type: RpcType, data: Schema) -> Self {
        let url = String::from("/")
            + match rpc_type {
                RpcType::Network => "network",
            };
        Self { url, data }
    }
}
