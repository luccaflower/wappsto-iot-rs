use dotenv;
use std::env;
use std::error::Error;
use wappsto_iot_rs::{create_network::*, network::*, test_await::aw};

#[test]
#[ignore]
fn publishes_new_network_to_wappsto() {
    create_network().expect("Failed to created network");

    let mut network: Network = Network::new("test").unwrap();
    assert!(aw!(network.start()).is_ok());
}

fn create_network() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv()?;
    let username = env::var("WAPPSTO_USERNAME")?;
    let password = env::var("WAPPSTO_PASSWORD")?;
    RequestBuilder::new()
        .with_credentials(&username, &password)
        .to_server(WappstoServers::QA)
        .send()?;
    Ok(())
}
