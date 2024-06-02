// Uncomment this block to pass the first stage
use anyhow::Result;
use itertools::Itertools;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

#[derive(Debug)]
struct Request<'a> {
    method: &'a str,
    target: &'a str,

    headers: Headers<'a>,
    body: &'a str,
}

#[derive(Debug)]
struct Headers<'a> {
    host: &'a str,
    user_agent: &'a str,
    accept: &'a str,
}

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
                let contents = contents.trim_end_matches('\0');
                let request = Request::parse_request(contents);

                if request.target == "/" {
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

impl<'a> Request<'a> {
    fn parse_request(req: &str) -> Request<'_> {
        let (request_line, headers_and_body) =
            req.split_once("\r\n").expect("request is not well formed.");

        let (method, target, _) = request_line
            .splitn(3, ' ')
            .collect_tuple()
            .expect("request is ill-formed");

        let (host, user_agent, accept, _, body) = headers_and_body
            .splitn(5, "\r\n")
            .collect_tuple()
            .expect("request is ill-formed");

        let host = &host[6..];
        let user_agent = &user_agent[12..];
        let accept = &accept[8..];

        let headers = Headers {
            host,
            user_agent,
            accept,
        };

        Request {
            method,
            target,
            headers,
            body,
        }
    }
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

fn echo(request: &str) -> &str {
    todo!()
}
