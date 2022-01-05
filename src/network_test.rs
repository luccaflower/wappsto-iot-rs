mod network {
    use std::str::FromStr;

    use uuid::Uuid;

    use crate::{
        fs_store::Store,
        network::{Device, Network},
        network_test::store::StoreMock,
        schema::{DeviceSchema, Schema},
        test_await::aw,
    };

    use super::{connection::ConnectionMock, store::DEFAULT_ID};

    #[test]
    fn should_start() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        assert!(aw!(network.start()).is_ok())
    }

    #[test]
    fn should_open_a_connection() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        aw!(network.start()).expect("Failed to start");
        assert!(network.connection().is_started);
    }

    #[test]
    fn should_load_certificates_on_start() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        aw!(network.start()).expect("Failed to start");
        assert_eq!(DEFAULT_ID, &network.id.to_string())
    }

    #[test]
    fn should_close_connnection_on_stop() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        aw!(network.start()).unwrap();
        network.stop().unwrap();
        assert!(&network.connection().was_closed);
    }

    #[test]
    fn should_save_schema_to_store_on_stop() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        aw!(network.start()).unwrap();
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
    fn should_load_schema_from_store_on_creation() {
        let mut schema = Schema::new("test", Uuid::from_str(&DEFAULT_ID).unwrap());
        let device = DeviceSchema::new("test_device", Uuid::new_v4());
        schema.device.push(device);
        let mut store = StoreMock::default();
        store.save_schema(schema).unwrap();
        println!("Printing all schemas:");
        store.schemas.iter().for_each(|(_, network)| {
            println!("Network: {}", network.name);
            network
                .device
                .iter()
                .for_each(|device| println!("Device: {}", device.name))
        });
        println!("----------------------");
        let mut network: Network<ConnectionMock, StoreMock> =
            Network::new_with_store("test", store);
        assert!(!network.devices().is_empty());
        assert!(network.devices().contains_key("test_device"))
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

    #[test]
    fn should_publish_itself_on_start() {
        let mut network: Network<ConnectionMock, StoreMock> = Network::new("test").unwrap();
        aw!(network.start()).unwrap();

        assert!(network.connection().received(&network.id.to_string()))
    }
}

pub mod device {

    use std::cell::RefCell;

    use crate::network::{Device, ValuePermission};

    #[test]
    fn should_create_new_value() {
        let mut device = Device::default();
        device.create_value("test", ValuePermission::R);
        assert!(device.values().get("test").is_some())
    }

    #[test]
    fn should_register_callback_on_writable_values() {
        let callback_was_called = RefCell::new(false);
        let mut device = Device::default();
        let callback = |_: String| {
            *callback_was_called.borrow_mut() = true;
        };
        let value = device.create_value("test_value", ValuePermission::RW(Box::new(callback)));
        value.control(String::new());

        assert!(*callback_was_called.borrow())
    }
}

pub mod connection {
    use crate::{
        certs::Certs,
        connection::{Connect, WappstoServers},
        rpc::Rpc,
    };
    use async_trait::async_trait;
    use std::error::Error;

    pub struct ConnectionMock {
        pub is_started: bool,
        pub was_closed: bool,
        received: String,
    }

    #[async_trait]
    impl Connect for ConnectionMock {
        fn new(_certs: Certs, _server: WappstoServers) -> Self {
            Self {
                is_started: false,
                was_closed: false,
                received: String::new(),
            }
        }

        async fn start(&mut self) -> Result<(), Box<dyn Error>> {
            self.is_started = true;
            Ok(())
        }

        fn stop(&mut self) {
            self.was_closed = true;
        }

        async fn send(&mut self, rpc: Rpc) {
            self.received
                .push_str(&serde_json::to_string(&rpc).unwrap())
        }
    }

    impl ConnectionMock {
        pub fn received(&self, term: &str) -> bool {
            self.received.contains(term)
        }
    }
}

pub mod store {
    use uuid::Uuid;

    use crate::{certs::Certs, fs_store::Store, schema::Schema};
    use std::{collections::HashMap, error::Error};
    pub const DEFAULT_ID: &str = "00000000-0000-0000-0000-000000000000";

    use openssl::{pkey::PKey, x509::X509};

    pub struct StoreMock {
        pub schemas: HashMap<Uuid, Schema>,
    }

    impl Store for StoreMock {
        fn load_certs(&self) -> Result<Certs, Box<dyn Error>> {
            Ok(Certs {
                id: Uuid::parse_str(DEFAULT_ID).unwrap(),
                ca: X509::builder().unwrap().build(),
                certificate: X509::builder().unwrap().build(),
                private_key: PKey::generate_x448().unwrap(),
            })
        }

        fn save_schema(&mut self, schema: Schema) -> Result<(), Box<dyn Error>> {
            println!("Saving schema with id: {}", schema.meta.id);
            let id = schema.meta.id.clone();
            self.schemas.insert(schema.meta.id, schema);
            match self.schemas.get(&id) {
                Some(_) => println!("Retrievable"),
                None => println!("Not retrievable"),
            }
            Ok(())
        }

        fn load_schema(&self, id: Uuid) -> Option<Schema> {
            println!("Loading schema with id: {}", id);
            println!("All schema id's:");
            self.schemas
                .iter()
                .for_each(|(id, _)| println!("Id: {}", id));
            println!("----------------");
            let schema = self.schemas.get(&id);
            match schema {
                Some(_) => println!("Found!"),
                None => println!("Not found!"),
            }
            schema.cloned()
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
