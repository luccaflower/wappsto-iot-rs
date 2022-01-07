mod support {
    pub(crate) mod rest;
}
use std::cell::RefCell;

use support::rest::rest::{create_network, credentials, RestServer, RestSession};
use wappsto_iot_rs::network::{Network, ValuePermission};

#[test]
#[ignore = "not implemented"]
fn should_handle_incoming_control_state() {
    create_network().expect("Failed to create network");
    let callback_was_called = RefCell::new(false);
    let callback = |_| *callback_was_called.borrow_mut() = true;
    let mut network: Network =
        Network::new_at(wappsto_iot_rs::connection::WappstoServers::QA, "test").unwrap();
    let device = network.create_device("test_device");
    let value = device.create_value("test_value", ValuePermission::W(Box::new(callback)));
    let control_id = value.control.as_ref().unwrap().id.clone();
    let (username, password) = credentials();
    RestSession::new(&username, &password, RestServer::Qa)
        .control(control_id, "1")
        .unwrap();
    assert!(*callback_was_called.borrow())
}
