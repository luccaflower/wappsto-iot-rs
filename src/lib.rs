//!A library to support third party development of software that integrates with an external IoT
//!service via JSON-RPC

///Create a new network and acquire its certificates and UUID
pub mod create_network;

///Data store for network schematics
pub mod fs_store;

///SSL Certificates used with Wappsto
pub mod certs;

///SSL Connection abstraction layer
pub mod connection;

///The main entry point for the user. Manages creation of network, devices, and values.
pub mod network;

mod rpc;
mod schema;

#[cfg(test)]
mod network_test;

#[cfg(test)]
pub mod test_await;
