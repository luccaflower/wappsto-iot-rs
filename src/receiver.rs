use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf},
    runtime::Runtime,
};
use uuid::Uuid;

use crate::rpc::{RpcData, RpcRequest};

pub fn start<T>(
    callbacks: HashMap<Uuid, Box<dyn FnMut(String) + Send + Sync>>,
    mut read: ReadHalf<T>,
    mut write: WriteHalf<T>,
) -> Sender<String>
where
    T: AsyncRead + AsyncWrite + Send + 'static,
{
    let rt = Runtime::new().unwrap();
    let handle_read = rt.handle().clone();
    let handle_write = rt.handle().clone();
    let (send, receive): (Sender<String>, Receiver<String>) = mpsc::channel();
    thread::spawn(move || loop {
        let msg = receive.recv().unwrap();
        handle_write.block_on(async { write.write_all(msg.as_bytes()).await.unwrap() });
    });

    let callbacks = Arc::new(Mutex::new(callbacks));
    let callbacks = callbacks.clone();

    thread::spawn(move || {
        let mut buf = [0; 4096];
        loop {
            let mut bytes = 0;
            handle_read.block_on(async { bytes = read.read(&mut buf).await.unwrap() });
            let data: RpcRequest = serde_json::from_slice(&buf[..bytes]).unwrap();
            #[allow(clippy::single_match)]
            match data.params.data {
                RpcData::Data(d) => callbacks.lock().unwrap().get_mut(&d.meta.id).unwrap()(d.data),
                _ => (),
            };
        }
    });
    send
}
