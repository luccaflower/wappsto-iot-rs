use dotenv;
use std::{collections::HashMap, env};
use wappsto_iot_rs::connection::Connect;
use wappsto_iot_rs::create_network::{RequestBuilder, WappstoServers};
use wappsto_iot_rs::{certs::Certs, connection::Connection};

mod support {
    pub(crate) mod aw;
}

#[test]
fn connects_to_wappsto() {
    dotenv::dotenv().ok();
    let username =
        env::var("WAPPSTO_USERNAME").expect("Wappsto username not found in environment variables");
    let password =
        env::var("WAPPSTO_PASSWORD").expect("Wappsto password not found in environment variables");

    let creator = RequestBuilder::new()
        .to_server(WappstoServers::QA)
        .with_credentials(&username, &password)
        .send()
        .unwrap();

    let certs = Certs::new(&creator.ca, &creator.certificate, &creator.private_key);

    assert!(Connection::new(
        certs.unwrap(),
        wappsto_iot_rs::connection::WappstoServers::QA,
    )
    .start(HashMap::new())
    .is_ok());
}
