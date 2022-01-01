//!A library to support third party development of software that integrates with an external IoT
//!service via JSON-RPC

///Create a new network and acquire its certificates and UUID
pub mod create_network;

///Represents the internal data structure used by Wappsto
pub mod schema;

///Data store for network schematics
pub mod fs_store;

pub mod certs;
pub mod connection;
pub mod network;
pub mod rpc;

#[cfg(test)]
mod network_test;
pub mod test_await;
