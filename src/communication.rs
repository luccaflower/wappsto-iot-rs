use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, sleep},
    time::Duration,
};
use uuid::Uuid;

use crate::rpc::{RpcData, RpcRequest};

type CallbackMap = HashMap<Uuid, Box<dyn FnMut(String) + Send + Sync>>;

pub fn start<T>(callbacks: CallbackMap, stream: T) -> Arc<Sender<String>>
where
    T: Read + Write + Send + 'static,
{
    let (send, receive): (Sender<String>, Receiver<String>) = mpsc::channel();
    let stream = Arc::new(Mutex::new(stream));
    let write = Arc::clone(&stream);
    let read = Arc::clone(&stream);
    let send = Arc::new(send);
    let _send_from_reader = send.clone();
    println!("spawn threads");
    thread::spawn(move || write_thread(write, receive));

    let callbacks = Arc::new(Mutex::new(callbacks));
    let callbacks = callbacks.clone();

    thread::spawn(move || {
        read_thread_sync(callbacks, read);
    });
    send
}

fn read_thread_sync<T>(callbacks: Arc<Mutex<CallbackMap>>, read: Arc<Mutex<T>>)
where
    T: Read + Write + Send + 'static,
{
    let mut buf = [0; 4096];
    loop {
        let bytes = read_all_from(&read, &mut buf);
        println!(
            "buf: {}",
            &buf[..bytes].iter().map(|c| *c as char).collect::<String>()
        );
        let data: RpcRequest = serde_json::from_slice(&buf[..bytes]).unwrap();
        #[allow(clippy::single_match)]
        match data.params.data {
            RpcData::Data(d) => callbacks.lock().unwrap().get_mut(&d.meta.id).unwrap()(d.data),
            _ => (),
        };
    }
}
fn read_all_from<T: Read>(reader: &Arc<Mutex<T>>, mut buf: &mut [u8]) -> usize {
    loop {
        let read = reader.lock().unwrap().read(&mut buf);
        match read {
            Ok(v) => break v,
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => sleep(Duration::from_millis(100)),
            Err(_) => (),
        }
    }
}

fn write_thread<T>(write: Arc<Mutex<T>>, receive: Receiver<String>)
where
    T: Write + Send + 'static,
{
    loop {
        let msg = receive.recv().unwrap();
        println!("received message: {}", &msg);
        write_all_to(&write, msg.as_bytes());
    }
}

fn write_all_to<T: Write>(writer: &Arc<Mutex<T>>, msg: &[u8]) {
    loop {
        let write = writer.lock().unwrap().write_all(msg);
        match write {
            Ok(_) => break,
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => sleep(Duration::from_millis(100)),
            Err(_) => break,
        }
    }
}
