mod request;
mod response;

pub use request::*;
pub use response::*;

use smol::{io::AsyncReadExt, net::TcpStream, stream::StreamExt};

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum HttpMethod {
    GET,
    PUT,
    POST,
    DELETE,
    OPTIONS,
    TRACE,
    INVALID,
}

impl From<&str> for HttpMethod {
    fn from(value: &str) -> HttpMethod {
        use HttpMethod::*;
        match value.to_ascii_uppercase().as_str() {
            "GET" => GET,
            "PUT" => PUT,
            "POST" => POST,
            "DELETE" => DELETE,
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
pub(crate) async fn read_header(
    stream: &mut TcpStream,
    buff: &mut Vec<u8>,
) -> std::io::Result<usize> {
    let bytes = &mut stream.bytes();
    let mut header_count = usize::MAX; //adjust for overcounting

    while let Some(byte) = bytes.next().await {
        buff.push(byte?);

        if buff.len() < 4 {
            continue;
        }

        if buff[buff.len() - 2..] == *b"\r\n" {
            header_count = header_count.wrapping_add(1);
        }

        if buff[buff.len() - 4..] == *b"\r\n\r\n" {
            break;
        }
    }
    Ok(header_count)
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
