mod network {
    use std::{
        str::FromStr,
        sync::{Arc, Mutex},
        thread::sleep,
        time::Duration,
    };

    use chrono::Utc;
    use uuid::Uuid;

    use crate::{
        fs_store::Store,
        network::{Network, ValuePermission},
        network_test::{connection::WrappedSendMock, store::StoreMock},
        rpc::{RpcData, RpcMethod, RpcRequest, RpcStateData},
        schema::{DeviceSchema, Meta, MetaType, Schema},
    };

    use super::{connection::ConnectionMock, store::DEFAULT_ID};

    #[test]
    fn should_start() {
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new("test").unwrap();
        assert!(network.start().is_ok())
    }

    #[test]
    fn should_open_a_connection() {
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new("test").unwrap();
        network.start().expect("Failed to start");
        assert!(*network.connection().is_started.borrow());
    }

    #[test]
    fn should_load_certificates_on_start() {
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new("test").unwrap();
        network.start().expect("Failed to start");
        assert_eq!(DEFAULT_ID, &network.id().to_string())
    }

    #[test]
    fn should_save_schema_to_store_on_stop() {
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new("test").unwrap();
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
    fn should_load_schema_from_store_on_creation() {
        let mut schema = Schema::new("test", Uuid::from_str(&DEFAULT_ID).unwrap());
        let device = DeviceSchema::new("test_device", Uuid::new_v4());
        schema.device.push(device);
        let store = StoreMock::default();
        store.save_schema(schema).unwrap();
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new_with_store("test", store);
        assert!(!network.devices_is_empty());
        assert!(network.device_named("test_device").is_some())
    }

    #[test]
    fn should_create_new_device() {
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new("test").unwrap();

        network.create_device("test device");
        assert!(network.device_named("test device").is_some())
    }

    #[test]
    fn should_load_existing_device_from_store() {
        let store = StoreMock::default();
        let mut schema = Schema::new("test", Uuid::parse_str(DEFAULT_ID).unwrap());
        let device = DeviceSchema::new("test_device", Uuid::new_v4());
        let device_id = device.meta.id.clone();
        schema.device.push(device);
        store.save_schema(schema).unwrap();
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new_with_store("test", store);
        assert_eq!(
            device_id,
            network.create_device("test_device").inner.borrow().id
        )
    }

    #[test]
    fn should_create_multiple_devices() {
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new("test").unwrap();
        let _device_1 = network.create_device("stuff");
        let _device_2 = network.create_device("other_stuff");
    }

    #[test]
    fn should_publish_itself_on_start() {
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new("test").unwrap();
        network.start().unwrap();
        sleep(Duration::from_millis(50));
        assert!(network
            .inner
            .borrow()
            .send
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .sent_to_server(&network.id().to_string()))
    }

    #[test]
    fn should_pass_callbacks_to_reader() {
        let callback_was_called = Arc::new(Mutex::new(false));
        let callback_was_called_sent = Arc::clone(&callback_was_called);
        let callback = move |_: String| {
            *callback_was_called_sent.lock().unwrap() = true;
        };
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new("test").unwrap();
        let device = network.create_device("test_device");
        let state_id = device
            .create_value("test_value", ValuePermission::RW(Box::new(callback)))
            .control_id();

        network
            .connection()
            .stream
            .borrow_mut()
            .as_mut()
            .unwrap()
            .receive(&control_state_rpc("1", state_id));
        network.start().unwrap();
        network
            .inner
            .borrow()
            .send
            .lock()
            .unwrap()
            .as_ref()
            .unwrap();
        sleep(Duration::from_millis(50));
        assert!(*callback_was_called.lock().unwrap())
    }
    pub fn control_state_rpc(data: &str, id: Uuid) -> String {
        serde_json::to_string(
            &RpcRequest::builder()
                .method(RpcMethod::Put)
                .data(RpcData::Data(RpcStateData::new(
                    data,
                    Utc::now(),
                    Meta::new_with_uuid(id, MetaType::State),
                )))
                .create(),
        )
        .unwrap()
    }
}

pub mod device {

    use std::sync::{Arc, Mutex};

    use crate::{
        network::{Device, ValuePermission},
        network_test::connection::WrappedSendMock,
    };

    #[test]
    fn should_create_new_value() {
        let device: Device<WrappedSendMock> = Device::default();
        device.create_value("test", ValuePermission::R);
        assert!(device.value_named("test").is_some())
    }

    #[test]
    fn should_register_callback_on_writable_values() {
        let callback_was_called = Arc::new(Mutex::new(false));
        let callback_was_called_sent = Arc::clone(&callback_was_called);
        let device: Device<WrappedSendMock> = Device::default();
        let callback = move |_: String| {
            *callback_was_called_sent.lock().unwrap() = true;
        };
        let value = device.create_value("test_value", ValuePermission::RW(Box::new(callback)));
        value.inner.lock().unwrap().control(String::new());

        assert!(*callback_was_called.lock().unwrap())
    }
}

pub mod value {
    use std::{sync::Arc, thread::sleep, time::Duration};

    use crate::{
        network::{Network, ValuePermission},
        network_test::network::control_state_rpc,
    };

    use super::{
        connection::{ConnectionMock, WrappedSendMock},
        store::StoreMock,
    };

    #[test]
    fn should_report_state_change_to_server() {
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new("test").unwrap();
        let device = network.create_device("test device");
        let value = device.create_value("test value", ValuePermission::R);
        network.start().unwrap();
        value.report("test report");
        assert!(network
            .inner
            .borrow()
            .send
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .sent_to_server("test report"))
    }

    #[test]
    fn should_reference_value_in_callback() {
        let network: Network<ConnectionMock, StoreMock, WrappedSendMock> =
            Network::new("test").unwrap();
        let device = network.create_device("test device");
        let value = device.create_value("test value", ValuePermission::RW(Box::new(|_| {})));
        let value_arc = Arc::clone(&value.inner);
        value.on_control(Box::new(move |data: String| {
            value_arc.lock().unwrap().report(&data)
        }));
        let state_id = value.control_id();
        network
            .connection()
            .stream
            .borrow_mut()
            .as_mut()
            .unwrap()
            .receive(&control_state_rpc("test report", state_id));
        network.start().unwrap();
        sleep(Duration::from_millis(50));

        assert!(network
            .inner
            .borrow()
            .send
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .sent_to_server("test report"))
    }
}

pub mod connection {
    use crate::{
        certs::Certs,
        communication::{self, CallbackMap},
        connection::{Connect, WappstoServers, WrappedSend},
        stream_mock::StreamMock,
    };
    use std::{cell::RefCell, error::Error, sync::mpsc::Sender};

    pub struct ConnectionMock {
        pub is_started: RefCell<bool>,
        pub was_closed: bool,
        pub stream: RefCell<Option<StreamMock>>,
    }

    impl Connect<WrappedSendMock> for ConnectionMock {
        fn new(_certs: Certs, _server: WappstoServers) -> Self {
            Self {
                is_started: RefCell::new(false),
                was_closed: false,
                stream: RefCell::new(Some(StreamMock::new())),
            }
        }

        fn start(&self, callbacks: CallbackMap) -> Result<WrappedSendMock, Box<dyn Error>> {
            *self.is_started.borrow_mut() = true;
            Ok(WrappedSendMock::new(communication::start(
                callbacks,
                self.stream.borrow_mut().take().unwrap(),
            )))
        }
    }

    pub struct WrappedSendMock {
        received: RefCell<String>,
        send: Sender<String>,
    }

    impl WrappedSendMock {
        pub fn new(send: Sender<String>) -> Self {
            Self {
                received: RefCell::new(String::new()),
                send,
            }
        }

        pub fn sent_to_server(&self, term: &str) -> bool {
            self.received.borrow().contains(term)
        }
    }
    impl WrappedSend for WrappedSendMock {
        fn send(&self, msg: String) -> Result<(), Box<dyn Error>> {
            self.received.borrow_mut().push_str(&msg);
            self.send.send(msg).unwrap();
            Ok(())
        }
    }
}

pub mod store {
    use uuid::Uuid;

    use crate::{certs::Certs, fs_store::Store, schema::Schema};
    use std::{cell::RefCell, collections::HashMap, error::Error};
    pub const DEFAULT_ID: &str = "00000000-0000-0000-0000-000000000000";

    use openssl::{pkey::PKey, x509::X509};

    pub struct StoreMock {
        pub schemas: RefCell<HashMap<Uuid, Schema>>,
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

        fn save_schema(&self, schema: Schema) -> Result<(), Box<dyn Error>> {
            self.schemas.borrow_mut().insert(schema.meta.id, schema);

            Ok(())
        }

        fn load_schema(&self, id: Uuid) -> Option<Schema> {
            self.schemas.borrow().get(&id).cloned()
        }
    }

    impl Default for StoreMock {
        fn default() -> Self {
            StoreMock {
                schemas: RefCell::new(HashMap::new()),
            }
        }
    }
}
