use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use smol::{io::AsyncReadExt, net::TcpStream, stream::StreamExt};

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum HttpMethod {
    GET,
    PUT,
    POST,
    DELETE,
    OPTION,
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
            "OPTION" => OPTION,
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
pub async fn read_header(stream: &mut TcpStream, buff: &mut Vec<u8>) -> std::io::Result<usize> {
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

pub struct Request<T = ()> {
    pub method: HttpMethod,
    pub headers: HashMap<String, Vec<String>>,
    pub body: T,
    pub(crate) readable: std::pin::Pin<Box<dyn smol::io::AsyncRead + Send>>,
}

impl Deref for Request {
    type Target = std::pin::Pin<Box<dyn smol::io::AsyncRead + Send>>;

    fn deref(&self) -> &Self::Target {
        &self.readable
    }
}

impl DerefMut for Request {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.readable
    }
}

pub struct Response {
    pub(crate) writeable: std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send>>,
}

impl Deref for Response {
    type Target = std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send>>;

    fn deref(&self) -> &Self::Target {
        &self.writeable
    }
}

impl DerefMut for Response {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writeable
    }
}
