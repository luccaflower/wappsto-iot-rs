use dotenv;
use std::env;
use std::net::TcpStream;
use wappsto_iot_rs::connection;
use wappsto_iot_rs::create_network::{RequestBuilder, WappstoServers};

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
        .send();

    assert!(connection::start::<TcpStream>(creator.expect("Error getting creator")).is_ok())
}
