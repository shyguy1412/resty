use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use serde::Serialize;
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
    pub fn new(request: httparse::Request<'a, 'a>, stream: TcpStream) -> Option<Self> {
        let Some(data) = request
            .method
            .zip(request.path)
            .zip(request.version)
            .map(|((a, b), c)| (a, b, c))
        else {
            return None;
        };

        Some(Self {
            method: data.0,
            path: data.1,
            version: data.2,
            headers: request.headers,
            readable: stream.boxed_reader(),
        })
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ResponseState {
    Status,
    Header,
    Body,
}

#[doc(hidden)]
pub struct UnTyped;

pub struct Response<'a, B = UnTyped> {
    state: ResponseState,
    status: Option<u16>,
    writeable: std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send + 'a>>,
    body: PhantomData<B>,
}

impl<'a, B> Deref for Response<'a, B> {
    type Target = std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send + 'a>>;

    fn deref(&self) -> &Self::Target {
        &self.writeable
    }
}

impl<B> DerefMut for Response<'_, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writeable
    }
}

impl Response<'_> {
    pub(crate) fn new(stream: TcpStream) -> Self {
        Self {
            state: ResponseState::Status,
            status: None,
            writeable: stream.boxed_writer(),
            body: PhantomData,
        }
    }
}

impl<'a, B> Response<'a, B> {
    pub async fn status(&mut self, status: u16, reason: &str) -> u16 {
        if let Some(status) = self.status {
            return status;
        }

        if status < 100 || status >= 1000 || self.state != ResponseState::Status {
            return 0;
        }

        self.status.replace(status);

        let _ = self
            .write(format!("HTTP/1.1 {status} {reason}\r\n").as_bytes())
            .await;

        self.state = ResponseState::Header;
        status
    }
}

impl<'a, B: Serialize> Response<'a, B> {
    pub async fn send(&mut self, data: B) {
        if self.state == ResponseState::Body {
            self.status(200, "OK").await;
        }

        if self.state == ResponseState::Header {
            let _ = self.write("\r\n".as_bytes()).await;
        }

        self.state = ResponseState::Body;

        let data = &serde_json::to_string(&data).expect("Faulty JSON");

        let _ = self.write(data.as_bytes()).await;
    }
}

impl<'a, B: Serialize> From<Response<'a, UnTyped>> for Response<'a, B> {
    fn from(value: Response<'a, UnTyped>) -> Self {
        Self {
            state: value.state,
            status: value.status,
            writeable: value.writeable,
            body: PhantomData,
        }
    }
}
