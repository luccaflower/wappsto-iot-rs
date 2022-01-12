mod receiver {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        thread::sleep,
        time::Duration,
    };

    use crate::{
        communication,
        rpc::{RpcData, RpcMethod, RpcRequest, RpcStateData},
        schema::{Meta, MetaType},
        stream_mock::StreamMock,
    };
    use chrono::Utc;
    use uuid::Uuid;

    pub const DEFAULT_ID: &str = "00000000-0000-0000-0000-000000000000";

    #[test]
    fn should_callback_on_control() {
        let mut stream = StreamMock::new();
        stream.receive(&control_state_rpc(
            "1",
            Uuid::parse_str(DEFAULT_ID).unwrap(),
        ));
        let callback_was_called = Arc::new(Mutex::new(false));
        let callback_arc = Arc::clone(&callback_was_called);
        let callback = move |_: String| {
            *callback_arc.lock().unwrap() = true;
        };
        let mut callbacks: HashMap<Uuid, Box<dyn FnMut(String) + Send + Sync>> = HashMap::new();
        callbacks.insert(Uuid::parse_str(DEFAULT_ID).unwrap(), Box::new(callback));

        communication::start(callbacks, stream);
        sleep(Duration::from_millis(10));
        assert!(*callback_was_called.lock().unwrap())
    }

    fn control_state_rpc(data: &str, id: Uuid) -> String {
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
