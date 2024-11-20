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
        match parse_message(&buf[..n]) {
            Ok((RedisMessage::Arrays(vec), _))
                if vec
                    .first()
                    .filter(|s| {
                        if let RedisMessage::String(s) = s {
                            s.to_ascii_lowercase() == "echo"
                        } else {
                            false
                        }
                    })
                    .is_some() =>
            {
                if let RedisMessage::String(msg) = vec.get(1).unwrap() {
                    stream
                        .write_all(format! {"+{}\r\n", msg}.as_bytes())
                        .unwrap();
                }
            }
            Ok(_) => {
                stream.write_all(b"+PONG\r\n").unwrap();
            }
            Err(_) => todo!(),
        }
    }
}

#[derive(Debug)]
enum RedisMessage {
    String(String),
    Arrays(Vec<RedisMessage>),
}

fn parse_message(buf: &[u8]) -> Result<(RedisMessage, &[u8]), &'static str> {
    match buf[0] {
        b'+' => parse_simple_string(&buf[1..]),
        b'*' => parse_arrays(&buf[1..]),
        b'$' => parse_bulk_string(&buf[1..]),
        _ => todo!(),
    }
}

fn parse_arrays(buf: &[u8]) -> Result<(RedisMessage, &[u8]), &'static str> {
    let mut msg: RedisMessage;
    if let (Some(num), mut rest) = parse_word(buf) {
        let len = String::from_utf8_lossy(num)
            .to_string()
            .parse::<usize>()
            .unwrap();
        let mut arrays = Vec::new();
        for _ in 0..len {
            (msg, rest) = parse_message(rest)?;
            arrays.push(msg);
        }
        return Ok((RedisMessage::Arrays(arrays), rest));
    }
    Err("invalid arrays")
}

fn parse_bulk_string(buf: &[u8]) -> Result<(RedisMessage, &[u8]), &'static str> {
    if let (Some(num), rest) = parse_word(buf) {
        let len = String::from_utf8_lossy(num)
            .to_string()
            .parse::<usize>()
            .unwrap();
        return Ok((
            RedisMessage::String(String::from_utf8_lossy(&rest[..len]).to_string()),
            &rest[len + 2..],
        ));
    }
    Err("invalid bulk string")
}

fn parse_word(buf: &[u8]) -> (Option<&[u8]>, &[u8]) {
    if let Some(index) = buf
        .iter()
        .enumerate()
        .find(|(_, &c)| c == b'\r')
        .map(|(i, _)| i)
    {
        return (Some(&buf[..index]), &buf[index + 2..]);
    }
    (None, buf)
}

fn parse_simple_string(buf: &[u8]) -> Result<(RedisMessage, &[u8]), &'static str> {
    if let (Some(s), rest) = parse_word(buf) {
        return Ok((
            RedisMessage::String(String::from_utf8_lossy(s).to_string()),
            rest,
        ));
    }
    Err("invalid simple string")
}
