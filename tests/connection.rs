use dotenv;
use std::env;
use wappsto_iot_rs::create_network::{RequestBuilder, WappstoServers};
use wappsto_iot_rs::{connection, fs_store};

#[test]
#[ignore]
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

    fs_store::save_certs(creator).unwrap();

    assert!(connection::start().is_ok())
}
