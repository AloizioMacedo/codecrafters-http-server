// Uncomment this block to pass the first stage
use anyhow::Result;
use std::{io::Write, net::TcpListener};

fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                stream.write_all(ok().as_bytes())?;
                println!("{}", ok());
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
