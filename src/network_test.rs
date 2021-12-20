mod network {
    use uuid::Uuid;

    use crate::{
        network::{Device, Network},
        network_test::store::StoreMock,
    };

    use super::{connection::ConnectionMock, store::DEFAULT_ID};

    #[test]
    fn should_start() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        assert!(network.start().is_ok())
    }

    #[test]
    fn should_open_a_connection() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        network.start().expect("Failed to start");
        assert!(network.connection().is_started);
    }

    #[test]
    fn should_load_certificates_on_startup() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        network.start().expect("Failed to start");
        assert_eq!(DEFAULT_ID, &network.id.to_string())
    }

    #[test]
    fn should_close_connnection_on_stop() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        network.start().unwrap();
        network.stop().unwrap();
        assert!(&network.connection().was_closed);
    }

    #[test]
    fn should_save_schema_to_store_on_stop() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        network.start().unwrap();
        network.stop().unwrap();
        assert_eq!(
            Uuid::parse_str(DEFAULT_ID).unwrap(),
            network
                .store()
                .load_schema(Uuid::parse_str(DEFAULT_ID).unwrap())
                .unwrap()
                .meta
                .id
        )
    }

    #[test]
    fn should_create_new_device() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();

        network.create_device("test device");
        assert!(network.devices().get("test device").is_some())
    }

    #[test]
    fn should_load_existing_device() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        let devices = network.devices();
        let device = Device::default();
        let expected_id = device.id.clone();
        devices.insert(String::from("test_device"), device);
        assert_eq!(expected_id, network.create_device("test_device").id)
    }

    #[test]
    fn should_create_multiple_devices() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        let _device_1 = network.create_device("stuff");
        let _device_2 = network.create_device("other_stuff");
    }
}

pub mod device {

    use crate::network::{Device, Value, ValuePermission};

    #[test]
    fn should_create_new_value() {
        let mut device = Device::default();
        device.create_value("test", ValuePermission::R);
        assert!(device.values().get("test").is_some())
    }

    #[test]
    #[ignore]
    fn should_register_callback_on_writable_values() {
        let mut callback_was_called = false;
        let mut device = Device::default();
        let callback = |_: String| {
            callback_was_called = true;
        };
        let value: &mut Value =
            device.create_value("test_value", ValuePermission::RW(Box::new(callback)));
        value.control(String::new());
        //FIX
        //assert!(callback_was_called)
    }
}

pub mod connection {
    use crate::{certs::Certs, connection::Connectable};
    use std::error::Error;

    pub struct ConnectionMock {
        pub is_started: bool,
        pub was_closed: bool,
    }

    impl Connectable for ConnectionMock {
        fn new(_certs: Certs) -> Self {
            Self {
                is_started: false,
                was_closed: false,
            }
        }
        fn start(&mut self) -> Result<(), Box<dyn Error>> {
            self.is_started = true;
            Ok(())
        }

        fn stop(&mut self) {
            self.was_closed = true;
        }
    }
}

pub mod store {
    use uuid::Uuid;

    use crate::{certs::Certs, fs_store::Store, schema::Schema};
    use std::{collections::HashMap, error::Error};
    pub const DEFAULT_ID: &str = "00000000-0000-0000-0000-000000000000";

    pub struct StoreMock {
        schemas: HashMap<Uuid, Schema>,
    }

    impl StoreMock {
        pub fn load_schema(&self, id: Uuid) -> Option<Schema> {
            let schema = self.schemas.get(&id).unwrap();
            Some(Schema::new("", schema.meta.id))
        }
    }

    impl Store for StoreMock {
        fn load_certs(&self) -> Result<Certs, Box<dyn Error>> {
            Ok(Certs {
                id: Uuid::parse_str(DEFAULT_ID).unwrap(),
                ca: String::from(""),
                certificate: String::from(""),
                private_key: String::from(""),
            })
        }

        fn save_schema(&mut self, schema: Schema) -> Result<(), Box<dyn Error>> {
            self.schemas.insert(schema.meta.id, schema);
            Ok(())
        }
    }

    impl Default for StoreMock {
        fn default() -> Self {
            StoreMock {
                schemas: HashMap::new(),
            }
        }
    }
}
