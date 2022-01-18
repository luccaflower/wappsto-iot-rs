use std::{thread::sleep, time::Duration};

mod support {
    pub(crate) mod rest;
}
use support::rest::rest::{create_network, credentials, RestServer, RestSession};
use wappsto_iot_rs::{connection::WappstoServers, network::*};

#[test]
#[ignore]
fn should_report_state_change_to_wappsto() {
    create_network().expect("Failed to create network");
    let network: Network = Network::new_at(WappstoServers::QA, "test").unwrap();
    let device = network.create_device("thing");
    let value = device.create_value("value", ValuePermission::R);
    let report_id = value.inner.report.as_ref().unwrap().id.clone();
    network.start().expect("Failed to start network");
    let (username, password) = credentials();
    value.report("5");
    sleep(Duration::from_secs(1));
    let report_value = RestSession::new(&username, &password, RestServer::Qa)
        .report(report_id)
        .unwrap();
    assert_eq!("5", &report_value);
    assert!(false)
}
