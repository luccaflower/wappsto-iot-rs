use serde_json::{Map, Value};
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

pub type CallbackMap = HashMap<Uuid, Arc<Mutex<Box<dyn FnMut(String) + Send + Sync>>>>;

pub fn start<T>(callbacks: CallbackMap, stream: T) -> Sender<String>
where
    T: Read + Write + Send + 'static,
{
    let stream = Arc::new(Mutex::new(stream));
    let write = Arc::clone(&stream);
    let read = Arc::clone(&stream);
    let (send, receive): (Sender<String>, Receiver<String>) = mpsc::channel();
    let send_from_reader = send.clone();
    thread::spawn(move || write_thread(write, receive));

    thread::spawn(move || {
        read_thread(callbacks, read, send_from_reader);
    });
    send
}

fn read_thread<T>(mut callbacks: CallbackMap, read: Arc<Mutex<T>>, _send: Sender<String>)
where
    T: Read + Write + Send + 'static,
{
    loop {
        let mut buf = [0; 4096];
        let bytes = read_all_from(&read, &mut buf);
        println!(
            "From server: {}",
            &buf[..bytes].iter().map(|c| *c as char).collect::<String>()
        );
        let data: Result<Map<String, Value>, _> = serde_json::from_slice(&buf[..bytes]);

        match data {
            Ok(d) if d.get("method").is_some() => {
                println!("got request!!");
                let data: RpcRequest = serde_json::from_slice(&buf[..bytes]).unwrap();
                #[allow(clippy::single_match)]
                match data.params.data {
                    RpcData::Data(d) => {
                        callbacks.get_mut(&d.meta.id).unwrap().lock().unwrap()(d.data)
                    }
                    _ => (),
                };
            }
            Ok(d) if d.get("result").is_some() => println!("got result"),
            Ok(d) => println!("Unknown message: {:?}", d),
            Err(e) => panic!("Deserialize error: {}", e),
        }
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
        println!("write thread: {}", msg);
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
