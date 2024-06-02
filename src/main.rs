// Uncomment this block to pass the first stage
use anyhow::{anyhow, Result};
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
    key_values: Vec<(&'a str, &'a str)>,
}

struct Response {
    code: u16,
    message: &'static str,
    body: String,
    headers: HeadersResponse,
}

#[derive(Debug)]
struct HeadersResponse {
    key_values: Vec<(&'static str, String)>,
}

impl Response {
    fn new(code: u16, message: &'static str) -> Response {
        Response {
            code,
            message,
            body: String::new(),
            headers: HeadersResponse { key_values: vec![] },
        }
    }

    fn with_headers(mut self, headers: Vec<(&'static str, String)>) -> Response {
        self.headers = HeadersResponse {
            key_values: headers,
        };

        self
    }

    fn with_body(mut self, body: String) -> Response {
        self.body = body;

        self
    }
}

impl From<Response> for String {
    fn from(value: Response) -> Self {
        let status_line = format!("HTTP/1.1 {} {}\r\n", value.code, value.message);

        let headers = value
            .headers
            .key_values
            .iter()
            .fold(String::new(), |acc, (k, v)| acc + k + ": " + v + "\r\n");

        status_line + &headers + "\r\n" + &value.body
    }
}

impl From<Response> for Vec<u8> {
    fn from(value: Response) -> Self {
        String::from(value).as_bytes().to_owned()
    }
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
                    let response = Response::new(200, "OK");
                    let response: Vec<u8> = response.into();

                    stream.write_all(&response).map_err(|e| {
                        eprintln!("{e}");
                        e
                    })?;
                } else if request.target.starts_with("/echo") {
                    let response = echo(&request)?;
                    let response: Vec<u8> = response.into();

                    stream.write_all(&response).map_err(|e| {
                        eprintln!("{e}");
                        e
                    })?;
                } else {
                    let response = Response::new(404, "Not Found");

                    let response: Vec<u8> = response.into();
                    stream.write_all(&response).map_err(|e| {
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

        let (headers, body) = headers_and_body
            .split_once("\r\n\r\n")
            .expect("request is ill-formed");

        let key_values = headers
            .split("\r\n")
            .map(|s| s.split_once(": ").expect("headers are ill-formed"))
            .collect();

        let headers = Headers { key_values };

        Request {
            method,
            target,
            headers,
            body,
        }
    }
}

fn echo(req: &Request) -> Result<Response> {
    let (_, _, content) = req
        .target
        .splitn(3, '/')
        .collect_tuple()
        .ok_or(anyhow!("invalid usage of /echo endpoint"))?;

    println!("Content for echo: {content}");

    let response = Response::new(200, "OK");

    Ok(response
        .with_headers(vec![
            ("Content-Type", "text/plain".to_string()),
            ("Content-Length", content.len().to_string()),
        ])
        .with_body(content.to_string()))
}
