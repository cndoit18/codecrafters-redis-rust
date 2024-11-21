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

fn main() {
    println!("starting server...");
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    let data = Arc::new(Mutex::new(HashMap::<String, String>::new()));
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
fn handle_connection(mut stream: TcpStream, mut data: Arc<Mutex<HashMap<String, String>>>) {
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
    data: &mut Arc<Mutex<HashMap<String, String>>>,
    stream: &mut TcpStream,
) {
    if v.is_empty() {
        return;
    }

    if let Value::String(s) = v.first().unwrap() {
        match s.to_lowercase().as_str() {
            "echo" => {
                if let Value::String(v) = v.get(1).unwrap() {
                    stream.write_all(format! {"+{}\r\n", v}.as_bytes()).unwrap();
                }
            }
            "set" => {
                if let Value::String(key) = v.get(1).unwrap() {
                    if let Value::String(value) = v.get(2).unwrap() {
                        *data
                            .lock()
                            .unwrap()
                            .entry(key.to_string())
                            .or_insert(value.to_string()) = value.to_string();
                        stream.write_all(b"+OK\r\n").unwrap();
                    }
                }
            }
            "get" => {
                if let Value::String(key) = v.get(1).unwrap() {
                    if data
                        .lock()
                        .unwrap()
                        .get(key)
                        .map(|v| stream.write_all(format! {"+{}\r\n",v}.as_bytes()).unwrap())
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
