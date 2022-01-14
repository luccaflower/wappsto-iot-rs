use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{Meta, Schema};

#[derive(Serialize, Deserialize)]
pub struct RpcRequest {
    jsonrpc: String,
    method: RpcMethod,
    id: String,
    pub params: RpcParams,
}

impl RpcRequest {
    pub fn builder() -> RpcRequestBuilder {
        RpcRequestBuilder::new()
    }

    pub fn new(method: RpcMethod, rpc_type: RpcType, data: RpcData) -> RpcRequest {
        Self {
            jsonrpc: String::from("2.0"),
            method,
            id: Uuid::new_v4().to_string(),
            params: RpcParams::new(rpc_type, data),
        }
    }
}

pub struct RpcRequestBuilder {
    method: RpcMethod,
    rpc_type: RpcType,
    data: RpcData,
}

impl RpcRequestBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            method: RpcMethod::Post,
            rpc_type: RpcType::Network,
            data: RpcData::None,
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

    pub fn data(mut self, data: RpcData) -> Self {
        self.data = data;
        self
    }

    pub fn create(self) -> RpcRequest {
        RpcRequest::new(self.method, self.rpc_type, self.data)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RpcMethod {
    Post,
    Put,
    Patch,
    Get,
    Delete,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RpcType {
    Network,
}

#[derive(Serialize, Deserialize)]
pub struct RpcParams {
    url: String,
    pub data: RpcData,
}

impl RpcParams {
    pub fn new(rpc_type: RpcType, data: RpcData) -> Self {
        let url = String::from("/")
            + match rpc_type {
                RpcType::Network => "network",
            };
        Self { url, data }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcData {
    Schema(Schema),
    Data(RpcStateData),
    None,
}

#[derive(Serialize, Deserialize)]
pub struct RpcStateData {
    pub data: String,
    timestamp: DateTime<Utc>,
    pub meta: Meta,
}

#[cfg(test)]
impl RpcStateData {
    pub fn new(data: &str, timestamp: DateTime<Utc>, meta: Meta) -> Self {
        Self {
            data: String::from(data),
            timestamp,
            meta,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RpcResponse {}
