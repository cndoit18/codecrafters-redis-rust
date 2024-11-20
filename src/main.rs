mod resp;

use resp::Interpreter;
use resp::Value;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    println!("starting server...");
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
fn handle_connection(mut stream: TcpStream) {
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
            Value::Array(l) if echo_command(l).is_some() => {
                stream
                    .write_all(echo_command(l).unwrap().as_bytes())
                    .unwrap();
            }
            _ => {
                stream.write_all(b"+PONG\r\n").unwrap();
            }
        });
    }
}

fn echo_command(v: &[Value]) -> Option<&str> {
    if v.len() != 2 {
        return None;
    }

    if let Value::String(s) = v.first().unwrap() {
        if s.to_lowercase() != "echo" {
            return None;
        }
        if let Value::String(v) = v.get(1).unwrap() {
            return Some(v.as_str());
        }
    }
    None
}
