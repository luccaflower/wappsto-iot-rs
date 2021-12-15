mod network {
    use crate::{network::Network, network_test::store::StoreMock};

    use super::{connection::Connection, store::DEFAULT_ID};

    #[test]
    fn should_start() {
        let mut network: Network<Connection, StoreMock> = Network::new("test").unwrap();
        assert!(network.start().is_ok())
    }

    #[test]
    fn should_open_a_connection() {
        let mut network: Network<Connection, StoreMock> = Network::new("test").unwrap();
        network.start().expect("Failed to start");
        assert!(network.connection().is_started);
    }

    #[test]
    fn should_load_certificates() {
        let mut network: Network<Connection, StoreMock> = Network::new("test").unwrap();
        network.start().expect("Failed to start");
        assert_eq!(DEFAULT_ID, &network.id.to_string())
    }
}

pub mod connection {
    use crate::connection::Connectable;
    use std::error::Error;

    pub struct Connection {
        pub is_started: bool,
    }

    impl Connection {
        pub fn new() -> Self {
            Self { is_started: false }
        }
    }

    impl Default for Connection {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Connectable for Connection {
        fn start(&mut self) -> Result<(), Box<dyn Error>> {
            self.is_started = true;
            Ok(())
        }
    }
}

pub mod store {
    use uuid::Uuid;

    use crate::{certs::Certs, fs_store::Store};
    use std::error::Error;
    pub const DEFAULT_ID: &str = "00000000-0000-0000-0000-000000000000";

    pub struct StoreMock;

    impl Store for StoreMock {
        fn load_certs(&self) -> Result<Certs, Box<dyn Error>> {
            Ok(Certs {
                id: Uuid::parse_str(DEFAULT_ID).unwrap(),
            })
        }
    }

    impl Default for StoreMock {
        fn default() -> Self {
            StoreMock {}
        }
    }
}
