mod request;
mod response;

use flate2::{write::GzEncoder, Compression};
use request::{Headers, Request};
use response::Response;

// Uncomment this block to pass the first stage
use anyhow::{anyhow, Result};
use itertools::Itertools;
use rayon::prelude::*;
use std::{
    env,
    fs::{read_to_string, write},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let args: Vec<String> = env::args().collect();

    let directory = if args.len() >= 2 && &args[1] == "--directory" {
        Some(PathBuf::from(&args[2]))
    } else {
        None
    };

    let directory = directory.as_deref();

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    listener.incoming().par_bridge().for_each(|stream| {
        _ = handle_stream(stream, directory);
    });

    Ok(())
}

fn handle_stream(
    stream: Result<TcpStream, std::io::Error>,
    directory: Option<&Path>,
) -> Result<()> {
    match stream {
        Ok(mut stream) => {
            println!("accepted new connection");
            let mut buf = [0; 1024];
            _ = stream.read(&mut buf)?;

            let contents = String::from_utf8_lossy(&buf);
            let contents = contents.trim_end_matches('\0');
            let Ok(request) = Request::parse_request(contents) else {
                let response = Response::new(500, "Internal Server Error");
                let response: Vec<u8> = response.into();

                stream.write_all(&response).map_err(|e| {
                    eprintln!("{e}");
                    e
                })?;

                return Ok(());
            };

            if request.target == "/" {
                let response = Response::new(200, "OK");
                let response: Vec<u8> = response.into();

                stream.write_all(&response).map_err(|e| {
                    eprintln!("{e}");
                    e
                })?;
            } else if request.target.starts_with("/echo") {
                let Ok(response) = echo(&request) else {
                    let response = Response::new(500, "Internal Server Error");
                    let response: Vec<u8> = response.into();

                    stream.write_all(&response).map_err(|e| {
                        eprintln!("{e}");
                        e
                    })?;
                    return Ok(());
                };

                let response: Vec<u8> = response.into();

                stream.write_all(&response).map_err(|e| {
                    eprintln!("{e}");
                    e
                })?;
            } else if request.target == "/user-agent" {
                let Ok(response) = user_agent(&request) else {
                    let response = Response::new(500, "Internal Server Error");
                    let response: Vec<u8> = response.into();

                    stream.write_all(&response).map_err(|e| {
                        eprintln!("{e}");
                        e
                    })?;
                    return Ok(());
                };

                let response: Vec<u8> = response.into();
                stream.write_all(&response).map_err(|e| {
                    eprintln!("{e}");
                    e
                })?;
            } else if request.target.starts_with("/files") {
                if request.method == "GET" {
                    let Ok(response) = files(&request, directory) else {
                        let response = Response::new(500, "Internal Server Error");
                        let response: Vec<u8> = response.into();

                        stream.write_all(&response).map_err(|e| {
                            eprintln!("{e}");
                            e
                        })?;
                        return Ok(());
                    };

                    let response: Vec<u8> = response.into();
                    stream.write_all(&response).map_err(|e| {
                        eprintln!("{e}");
                        e
                    })?;
                } else if request.method == "POST" {
                    let Ok(response) = files_post(&request, directory) else {
                        let response = Response::new(500, "Internal Server Error");
                        let response: Vec<u8> = response.into();

                        stream.write_all(&response).map_err(|e| {
                            eprintln!("{e}");
                            e
                        })?;
                        return Ok(());
                    };

                    let response: Vec<u8> = response.into();
                    stream.write_all(&response).map_err(|e| {
                        eprintln!("{e}");
                        e
                    })?;
                }
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

    Ok(())
}

impl<'a> Request<'a> {
    fn parse_request(req: &str) -> Result<Request<'_>> {
        let (request_line, headers_and_body) = req
            .split_once("\r\n")
            .ok_or(anyhow!("request is not well formed."))?;

        let (method, target, _) = request_line
            .splitn(3, ' ')
            .collect_tuple()
            .ok_or(anyhow!("request is ill-formed"))?;

        if let Some(body) = headers_and_body.strip_prefix("\r\n") {
            return Ok(Request {
                method,
                target,
                headers: Headers { key_values: vec![] },
                body,
            });
        }

        let (headers, body) = headers_and_body
            .split_once("\r\n\r\n")
            .ok_or(anyhow!("request is ill-formed"))?;

        let key_values = headers
            .split("\r\n")
            .map(|s| s.split_once(": ").ok_or(anyhow!("headers are ill-formed")))
            .collect::<Result<Vec<_>>>()?;

        let headers = Headers { key_values };

        Ok(Request {
            method,
            target,
            headers,
            body,
        })
    }
}

fn echo(req: &Request) -> Result<Response> {
    let (_, _, content) = req
        .target
        .splitn(3, '/')
        .collect_tuple()
        .ok_or(anyhow!("invalid usage of /echo endpoint"))?;

    println!("Content for echo: {content}");

    let encoding = req.headers.key_values.iter().find_map(|(k, v)| {
        (k == &"Accept-Encoding" && get_encodings(v).contains(&"gzip")).then_some("gzip")
    });

    let response = Response::new(200, "OK");

    if let Some(encoding) = encoding {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(content.as_bytes())?;
        let compressed = encoder.finish()?;

        Ok(response
            .with_headers(vec![
                ("Content-Type", "text/plain".to_string()),
                ("Content-Length", compressed.len().to_string()),
                ("Content-Encoding", encoding.to_string()),
            ])
            .with_body(compressed))
    } else {
        Ok(response
            .with_headers(vec![
                ("Content-Type", "text/plain".to_string()),
                ("Content-Length", content.len().to_string()),
            ])
            .with_body(content.as_bytes().to_vec()))
    }
}

fn get_encodings(v: &str) -> impl Iterator<Item = &str> {
    v.split(", ")
}

fn user_agent(req: &Request) -> Result<Response> {
    let user_agent = req
        .headers
        .key_values
        .iter()
        .find_map(|(k, v)| if k == &"User-Agent" { Some(v) } else { None })
        .ok_or(anyhow!("user-agent not found"))?;

    let response = Response::new(200, "OK");

    Ok(response
        .with_headers(vec![
            ("Content-Type", "text/plain".to_string()),
            ("Content-Length", user_agent.len().to_string()),
        ])
        .with_body(user_agent.as_bytes().to_vec()))
}

fn files(req: &Request, directory: Option<&Path>) -> Result<Response> {
    let Some(directory) = directory else {
        return Err(anyhow!("directory not passed to /files endpoint"));
    };
    let (_, _, filename) = req
        .target
        .splitn(3, '/')
        .collect_tuple()
        .ok_or(anyhow!("invalid usage of /files endpoint"))?;

    let Ok(contents) = read_to_string(directory.join(filename)) else {
        let response = Response::new(404, "Not Found");

        return Ok(response);
    };

    let response = Response::new(200, "OK");

    Ok(response
        .with_headers(vec![
            ("Content-Type", "application/octet-stream".to_string()),
            ("Content-Length", contents.len().to_string()),
        ])
        .with_body(contents.as_bytes().to_vec()))
}

fn files_post(req: &Request, directory: Option<&Path>) -> Result<Response> {
    let Some(directory) = directory else {
        return Err(anyhow!("directory not passed to /files endpoint"));
    };
    let (_, _, filename) = req
        .target
        .splitn(3, '/')
        .collect_tuple()
        .ok_or(anyhow!("invalid usage of /files endpoint"))?;

    let body = req.body;

    write(directory.join(filename), body)?;

    let response = Response::new(201, "Created");

    Ok(response)
}
