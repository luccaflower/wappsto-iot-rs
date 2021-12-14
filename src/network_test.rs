mod network {
    use crate::{network::Network, network_test::connection::NetworkMock};

    use super::connection::Connection;

    #[test]
    fn should_start() {
        let network_result = Network::new("test").start();
        assert!(network_result.is_ok())
    }

    #[test]
    fn should_open_a_connection() {
        let connection = Connection::new();
        let mut network = Network::new_mock("test", connection);
        network.start().expect("Failed to start");
        assert!(network.connection.is_started);
    }
}

pub mod connection {
    use crate::{connection::Connectable, network::Network};
    use std::error::Error;

    pub struct Connection {
        pub is_started: bool,
    }

    impl Connection {
        pub fn new() -> Self {
            Self { is_started: false }
        }
    }

    impl Connectable for Connection {
        fn start(&mut self) -> Result<(), Box<dyn Error>> {
            self.is_started = true;
            Ok(())
        }
    }

    pub trait NetworkMock<'a> {
        fn new_mock(name: &'a str, connection: Connection) -> Self;
    }

    impl<'a> NetworkMock<'a> for Network<Connection> {
        fn new_mock(_name: &'a str, connection: Connection) -> Self {
            Self {
                connection: connection,
            }
        }
    }
}
