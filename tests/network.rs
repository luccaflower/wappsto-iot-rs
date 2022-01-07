use wappsto_iot_rs::{connection::WappstoServers, network::*};

mod support {
    pub(crate) mod aw;
    pub(crate) mod rest;
}
use support::rest::create_network;

#[test]
fn publishes_new_network_to_wappsto() {
    create_network().expect("Failed to create network");
    let mut network: Network = Network::new_at(WappstoServers::QA, "test").unwrap();
    let device = network.create_device("thing");
    device.create_value("value", ValuePermission::RW(Box::new(|_| {})));
    assert!(aw!(network.start()).is_ok());
    assert!(network.stop().is_ok())
}
