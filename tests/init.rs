use dotenv;
use std::env;
use std::error::Error;
use wappsto_iot_rs::create_network::*;
use wappsto_iot_rs::network::*;

#[test]
#[ignore]
fn publishes_new_network_to_wappsto() {
    create_network().expect("Failed to created network");

    assert!(Network::builder("test").create().is_ok());
}

fn create_network() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv()?;
    let username = env::var("WAPPSTO_USERNAME")?;
    let password = env::var("WAPPSTO_PASSWORD")?;
    RequestBuilder::new()
        .to_server(WappstoServers::QA)
        .with_credentials(&username, &password)
        .send()?;
    Ok(())
}
