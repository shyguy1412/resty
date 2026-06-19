pub mod request;
pub mod response;

use smol::{io::AsyncReadExt, net::TcpStream, stream::StreamExt};

/// Enum for every valid HTTP method with a variant for invalid methods
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
#[repr(u8)]
pub enum HttpMethod {
    GET     = 0b00000001u8,
    PUT     = 0b00000010u8,
    POST    = 0b00000100u8,
    DELETE  = 0b00001000u8,
    PATCH   = 0b00010000u8,
    HEAD    = 0b00100000u8,
    OPTIONS = 0b01000000u8,
    TRACE   = 0b10000000u8,
    INVALID = 0b00000000u8,
}

impl From<&str> for HttpMethod {
    fn from(value: &str) -> HttpMethod {
        use HttpMethod::*;
        match value.to_ascii_uppercase().as_str() {
            "GET" => GET,
            "PUT" => PUT,
            "POST" => POST,
            "DELETE" => DELETE,
            "PATCH" => PATCH,
            "HEAD" => HEAD,
            "OPTIONS" => OPTIONS,
            "TRACE" => TRACE,
            _ => INVALID,
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[inline(always)]
pub(crate) async fn read_headers(stream: &mut TcpStream, buff: &mut Vec<u8>) -> Option<usize> {
    let bytes = &mut stream.bytes();
    let mut header_count: usize = 0; //adjust for overcounting
    while let Some(Ok(byte)) = bytes.next().await {
        buff.push(byte);

        if buff.len() < 4 {
            continue;
        }

        if buff[buff.len() - 2..] == *b"\r\n" {
            header_count = header_count.wrapping_add(1);
        }

        if buff[buff.len() - 4..] == *b"\r\n\r\n" {
            return Some(header_count);
        }
    }
    None
}

#[inline(always)]
pub(crate) fn parse_cookies<'a>(headers: &'a [httparse::Header]) -> Vec<(&'a str, &'a str)> {
    headers
        .iter()
        .filter_map(|h| match h.name == "Cookie" {
            true => str::from_utf8(h.value).ok(),
            false => None,
        })
        .flat_map(|value| value.split("; "))
        .filter_map(|cookie| cookie.split_once("="))
        .collect()
}
