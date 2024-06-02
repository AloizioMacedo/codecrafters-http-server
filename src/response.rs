pub struct Response {
    pub code: u16,
    pub message: &'static str,
    pub body: String,
    pub headers: HeadersResponse,
}

#[derive(Debug)]
pub struct HeadersResponse {
    pub key_values: Vec<(&'static str, String)>,
}

impl Response {
    pub fn new(code: u16, message: &'static str) -> Response {
        Response {
            code,
            message,
            body: String::new(),
            headers: HeadersResponse { key_values: vec![] },
        }
    }

    pub fn with_headers(mut self, headers: Vec<(&'static str, String)>) -> Response {
        self.headers = HeadersResponse {
            key_values: headers,
        };

        self
    }

    pub fn with_body(mut self, body: String) -> Response {
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
