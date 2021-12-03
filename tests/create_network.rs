use dotenv;
use std::env;
use wappsto_iot_rs::create_network::*;

#[test]
fn creates_network() {
    dotenv::dotenv().ok();
    let username =
        env::var("WAPPSTO_USERNAME").expect("Wappsto username not found in environment variables");
    let password =
        env::var("WAPPSTO_PASSWORD").expect("Wappsto password not found in environment variables");
    let response = RequestBuilder::new()
        .with_credentials(&username, &password)
        .to_server(WappstoServers::QA)
        .send();

    assert!(response.is_ok());
}
