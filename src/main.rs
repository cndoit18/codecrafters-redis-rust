mod resp;

use resp::Interpreter;
use resp::Value;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

fn main() {
    println!("starting server...");
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    let data = Arc::new(Mutex::new(HashMap::<String, Val>::new()));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let data = data.clone();
                thread::spawn(move || {
                    handle_connection(stream, data);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
fn handle_connection(mut stream: TcpStream, mut data: Arc<Mutex<HashMap<String, Val>>>) {
    let mut buf = [0_u8; 1024];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break;
        }
        println!(
            "received: {:?}",
            String::from_utf8_lossy(&buf[..n]).to_string()
        );

        let engine = Interpreter::new(&buf);
        let command = engine.parse();
        command.iter().for_each(|c| match c {
            Value::Array(l) => {
                command_handle(l, &mut data, &mut stream);
            }
            _ => {
                stream.write_all(b"+PONG\r\n").unwrap();
            }
        });
    }
}

fn command_handle(
    v: &[Value],
    data: &mut Arc<Mutex<HashMap<String, Val>>>,
    stream: &mut TcpStream,
) {
    if v.is_empty() {
        return;
    }

    if let Value::String(s) = v.first().unwrap() {
        match s.to_lowercase().as_str() {
            "echo" => {
                if let [_, Value::String(value)] = v {
                    stream
                        .write_all(format! {"+{}\r\n", value}.as_bytes())
                        .unwrap();
                }
            }
            "set" => {
                if let [_, Value::String(key), Value::String(value)] = v {
                    *data
                        .lock()
                        .unwrap()
                        .entry(key.to_string())
                        .or_insert(Val::String(value.to_string())) = Val::String(value.to_string());
                    stream.write_all(b"+OK\r\n").unwrap();
                }
                if let [_, Value::String(key), Value::String(value), Value::String(pt), Value::String(duration)] =
                    v
                {
                    if pt.to_ascii_lowercase() != "px" {
                        stream.write_all(b"$-1\r\n").unwrap();
                        return;
                    }
                    let now = SystemTime::now()
                        .checked_add(Duration::from_millis(duration.parse::<u64>().unwrap()))
                        .unwrap()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    println!("expire: {:?}", now);
                    *data
                        .lock()
                        .unwrap()
                        .entry(key.to_string())
                        .or_insert(Val::Expiry(value.to_string(), now)) =
                        Val::Expiry(value.to_string(), now);
                    stream.write_all(b"+OK\r\n").unwrap();
                }
            }
            "get" => {
                if let [_, Value::String(key)] = v {
                    if data
                        .lock()
                        .unwrap()
                        .get(key)
                        .map(|v| match v {
                            Val::String(s) => {
                                stream.write_all(format! {"+{}\r\n",s}.as_bytes()).unwrap();
                            }
                            Val::Expiry(s, duration)
                                if &SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis()
                                    < duration =>
                            {
                                stream.write_all(format! {"+{}\r\n",s}.as_bytes()).unwrap();
                            }
                            _ => {
                                stream.write_all(b"$-1\r\n").unwrap();
                            }
                        })
                        .is_none()
                    {
                        stream.write_all(b"$-1\r\n").unwrap();
                    };
                }
            }
            _ => {
                stream.write_all(b"+PONG\r\n").unwrap();
            }
        }
    }
}

enum Val {
    String(String),
    Expiry(String, u128),
}
