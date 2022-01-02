use dotenv;
use std::env;
use std::error::Error;
use wappsto_iot_rs::{
    certs::Certs, create_network::*, fs_store::FsStore, network::*, test_await::aw,
};

#[test]
fn publishes_new_network_to_wappsto() {
    create_network().expect("Failed to created network");

    let mut network: Network = Network::new("test").unwrap();
    assert!(aw!(network.start()).is_ok());
}

fn create_network() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv()?;
    let username = env::var("WAPPSTO_USERNAME")?;
    let password = env::var("WAPPSTO_PASSWORD")?;
    let creator = RequestBuilder::new()
        .with_credentials(&username, &password)
        .to_server(WappstoServers::QA)
        .send()?;
    let certs = Certs::new(&creator.ca, &creator.certificate, &creator.private_key).unwrap();
    FsStore::default().save_certs(certs).unwrap();
    Ok(())
}
