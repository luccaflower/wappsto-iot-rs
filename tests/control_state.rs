mod support {
    pub(crate) mod rest;
}
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use support::rest::rest::{create_network, credentials, RestServer, RestSession};
use wappsto_iot_rs::network::{OuterNetwork, ValuePermission};

#[test]
fn should_handle_incoming_control_state() {
    create_network().expect("Failed to create network");
    let callback_was_called = Arc::new(Mutex::new(false));
    let callback_was_called_sent = Arc::clone(&callback_was_called);
    let callback = move |_| *callback_was_called_sent.lock().unwrap() = true;
    let network: OuterNetwork =
        OuterNetwork::new_at(wappsto_iot_rs::connection::WappstoServers::QA, "test").unwrap();
    let device = network.create_device("test_device");
    let value = device.create_value("test_value", ValuePermission::W(Box::new(callback)));
    let control_id = value.inner.control.as_ref().unwrap().inner.id;
    network.start().unwrap();
    let (username, password) = credentials();
    sleep(Duration::from_secs(1));
    RestSession::new(&username, &password, RestServer::Qa)
        .control(control_id, "1")
        .unwrap();
    sleep(Duration::from_secs(1));
    assert!(*callback_was_called.lock().unwrap())
}
