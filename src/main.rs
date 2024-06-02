// Uncomment this block to pass the first stage
use anyhow::Result;
use itertools::Itertools;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buf = [0; 1024];
                _ = stream.read(&mut buf)?;

                let contents = String::from_utf8_lossy(&buf);
                let (_, path) = extract_full_url(&contents);

                if path == "/" {
                    stream.write_all(ok().as_bytes()).map_err(|e| {
                        eprintln!("{e}");
                        e
                    })?;
                } else {
                    stream.write_all(not_found().as_bytes()).map_err(|e| {
                        eprintln!("{e}");
                        e
                    })?;
                }

                stream.flush()?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}

fn ok() -> String {
    String::from("HTTP/1.1 200 OK\r\n\r\n")
}

fn not_found() -> String {
    String::from("HTTP/1.1 404 Not Found\r\n\r\n")
}

fn extract_full_url(request: &str) -> (&str, &str) {
    let (request_line, headers) = request
        .split_once("\r\n")
        .expect("request is not well formed.");

    let (_, path, _) = request_line
        .splitn(3, ' ')
        .collect_tuple()
        .expect("request is ill-formed");

    let (host, _) = headers
        .split_once("\r\n")
        .expect("request is not well formed");

    let host = &host[6..];

    (host, path)
}
