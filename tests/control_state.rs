mod support {
    pub(crate) mod rest;
}
/*use std::{cell::RefCell, env};

use support::rest::{create_network, credentials, RestServer, RestSession};
use wappsto_iot_rs::network::Network;

#[test]
fn should_handle_incoming_control_state() {

    let (username, password) = credentials();
    let session = RestSession::new(&username, &password, server, RestServer::Qa);
    create_network().expect("Failed to create network");
    let mut network: Network = Network::new_at(wappsto_iot_rs::connection::WappstoServers::QA, "test").unwrap();
    let device = network.create_device("test_device");
    let callback_was_called = RefCell::new(false);
    let callback = |_| { *callback_was_called.borrow_mut() = true };
    let value_id = device.create_value("test_value", ValuePermission::W(callback)).id;
}*/
