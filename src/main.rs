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
    loop {
        let _ = stream.read(&mut buf).unwrap();
        stream.write_all(b"+PONG\r\n").unwrap();
    }
}
