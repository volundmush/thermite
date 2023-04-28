use rand::{
    Rng,
    thread_rng,
    distributions::Alphanumeric
};

use std::{
    iter,
    collections::HashSet
};


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
    let mut state = 0;

    for &byte in data {
        match state {
            0 => {
                if byte == 0x16 {
                    state = 1;
                } else {
                    return ClientHelloStatus::Invalid;
                }
            }
            1 => {
                if byte >= 0x03 {
                    state = 2;
                } else {
                    return ClientHelloStatus::Invalid;
                }
            }
            2 => {
                if byte >= 0x01 {
                    state = 3;
                } else {
                    return ClientHelloStatus::Invalid;
                }
            }
            3 => {
                state = 4;
            }
            4 => {
                // At this point, we have a valid 5-byte sequence
                let handshake_length = u16::from_be_bytes([data[3], data[4]]) as usize;

                if data.len() >= handshake_length + 5 {
                    return ClientHelloStatus::Complete;
                } else {
                    return ClientHelloStatus::Partial;
                }
            }
            _ => unreachable!(),
        }
    }

    // If we reach this point, the data is a partial match
    ClientHelloStatus::Partial
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