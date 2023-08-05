use rand::{
    Rng,
    thread_rng,
    distributions::Alphanumeric
};

use std::{
    iter,
    collections::HashSet
};
use std::error::Error;
use std::net::SocketAddr;
use trust_dns_resolver::TokioAsyncResolver;


pub fn ensure_crlf(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_char_is_cr = false;

    for c in input.chars() {
        if c == '\n' && !prev_char_is_cr {
            result.push('\r');
        }
        prev_char_is_cr = c == '\r';
        result.push(c);
    }

    result
}

pub fn random_alphanum(length: usize) -> String {
    let mut rng = thread_rng();
    let chars: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(length)
        .collect();
    chars
}

pub fn repeat_string(src: &str, count: usize) -> String {
    iter::repeat(src).take(count).collect::<String>()
}

pub fn generate_id(count: usize, existing: &HashSet<String>) -> String {
    let mut new_id = random_alphanum(count);
    while existing.contains(&new_id) {
        new_id = random_alphanum(count);
    }
    new_id
}

#[derive(Clone, Debug)]
pub enum ClientHelloStatus {
    Complete,
    Partial,
    Invalid,
}

pub fn check_tls_client_hello(data: &[u8]) -> ClientHelloStatus {
    if data.len() < 5 {
        return ClientHelloStatus::Partial;
    }

    // Check for the TLS handshake type and version
    if data[0] != 0x16 || data[1] < 0x03 || data[2] < 0x01 {
        return ClientHelloStatus::Invalid;
    }

    ClientHelloStatus::Complete
}

pub enum HttpRequestStatus {
    Complete,
    Partial,
    Invalid,
}

static HTTP_METHODS: &[&[u8]] = &[
    b"GET ",
    b"POST ",
    b"PUT ",
    b"DELETE ",
    b"HEAD ",
    b"OPTIONS ",
    b"PATCH ",
    b"CONNECT ",
];

pub fn check_http_request(data: &[u8]) -> HttpRequestStatus {

    for method in HTTP_METHODS {
        if data.starts_with(method) {
            // Check if we have a full line
            if let Some(newline_pos) = data.iter().position(|&byte| byte == b'\n') {
                return HttpRequestStatus::Complete;
            } else {
                return HttpRequestStatus::Partial;
            }
        }
    }

    HttpRequestStatus::Invalid
}

pub async fn resolve_hostname(addr: SocketAddr) -> Result<Vec<String>, Box<dyn Error>> {
    let resolver = TokioAsyncResolver::tokio_from_system_conf()?;

    let response = resolver.reverse_lookup(addr.ip()).await?;
    let hostnames = response.iter().map(|x| x.to_string()).collect();

    Ok(hostnames)
}