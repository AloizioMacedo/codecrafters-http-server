#[derive(Debug)]
pub struct Request<'a> {
    pub method: &'a str,
    pub target: &'a str,

    pub headers: Headers<'a>,
    pub body: &'a str,
}

#[derive(Debug)]
pub struct Headers<'a> {
    pub key_values: Vec<(&'a str, &'a str)>,
}
