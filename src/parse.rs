use std::ops::{Deref, DerefMut};

use smol::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    stream::StreamExt,
};

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

pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub version: u8,
    pub headers: &'a mut [httparse::Header<'a>],
    readable: std::pin::Pin<Box<dyn smol::io::AsyncRead + Send>>,
}

impl<'a> Request<'a> {
    pub fn new(
        method: &'a str,
        path: &'a str,
        version: u8,
        headers: &'a mut [httparse::Header<'a>],
        stream: TcpStream,
    ) -> Self {
        Self {
            method,
            path,
            version,
            headers,
            readable: stream.boxed_reader(),
        }
    }
}

impl<'a> Deref for Request<'a> {
    type Target = std::pin::Pin<Box<dyn smol::io::AsyncRead + Send>>;

    fn deref(&self) -> &Self::Target {
        &self.readable
    }
}

impl DerefMut for Request<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.readable
    }
}

pub struct Response<'a> {
    writeable: std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send + 'a>>,
}

impl<'a> Deref for Response<'a> {
    type Target = std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send + 'a>>;

    fn deref(&self) -> &Self::Target {
        &self.writeable
    }
}

impl DerefMut for Response<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writeable
    }
}

impl<'a> Response<'a> {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            writeable: stream.boxed_writer(),
        }
    }
}
