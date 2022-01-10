mod receiver {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        thread::sleep,
        time::Duration,
    };

    use super::stream::StreamMock;
    use crate::{
        receiver,
        rpc::{RpcData, RpcMethod, RpcRequest, RpcStateData},
        schema::{Meta, MetaType},
    };
    use chrono::Utc;
    use tokio::io::split;
    use uuid::Uuid;

    pub const DEFAULT_ID: &str = "00000000-0000-0000-0000-000000000000";

    #[test]
    fn should_callback_on_control() {
        let mut stream = StreamMock::new();
        stream.receive(&control_state_rpc(
            "1",
            Uuid::parse_str(DEFAULT_ID).unwrap(),
        ));
        let (read, write) = split(stream);
        let callback_was_called = Arc::new(Mutex::new(false));
        let callback_arc = Arc::clone(&callback_was_called);
        let callback = move |_: String| {
            *callback_arc.lock().unwrap() = true;
        };
        let mut callbacks: HashMap<Uuid, Box<dyn FnMut(String) + Send + Sync>> = HashMap::new();
        callbacks.insert(Uuid::parse_str(DEFAULT_ID).unwrap(), Box::new(callback));

        receiver::start(callbacks, read, write);
        sleep(Duration::from_millis(50));
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

mod stream {

    use core::task::Poll;

    use tokio::io::{AsyncRead, AsyncWrite};

    pub struct StreamMock {
        in_buffer: String,
        out_buffer: String,
    }

    impl StreamMock {
        pub fn new() -> Self {
            Self {
                in_buffer: String::new(),
                out_buffer: String::new(),
            }
        }

        pub fn receive(&mut self, message: &str) {
            self.in_buffer.push_str(message)
        }
    }

    impl AsyncRead for StreamMock {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            if self.in_buffer.is_empty() {
                Poll::Pending
            } else {
                buf.put_slice(self.in_buffer.as_bytes());
                Poll::Ready(Ok(()))
            }
        }
    }

    impl AsyncWrite for StreamMock {
        fn poll_write(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            self.out_buffer
                .push_str(&buf.iter().map(|c| *c as char).collect::<String>());
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
        }
    }
}
