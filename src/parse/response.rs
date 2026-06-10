use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Copy)]
pub enum Error {
    SerializeError,
    WriteError,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SerializeError => write!(f, "SerializeError"),
            Error::WriteError => write!(f, "WriteError"),
        }
    }
}

use smol::io::AsyncWriteExt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ResponseState {
    Status,
    Header,
    Body,
}

pub struct Response<B = ()> {
    state: ResponseState,
    status: Option<u16>,
    writeable: std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send>>,
    body: PhantomData<B>,
}

impl<B> Deref for Response<B> {
    type Target = std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send>>;

    fn deref(&self) -> &Self::Target {
        &self.writeable
    }
}

impl<B> DerefMut for Response<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writeable
    }
}

impl<B> Response<B> {
    pub fn new(stream: smol::net::TcpStream) -> Self {
        Self {
            state: ResponseState::Status,
            status: None,
            writeable: stream.boxed_writer(),
            body: PhantomData,
        }
    }

    #[doc = "hidden"]
    pub async fn into_typed<T>(self) -> Response<T> {
        Response {
            state: self.state,
            status: self.status,
            writeable: self.writeable,
            body: PhantomData,
        }
    }
}

impl<B> Response<B> {
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

impl<B: Serialize> Response<B> {
    pub async fn send(&mut self, data: B) -> Result<(), Error> {
        if self.state == ResponseState::Body {
            self.status(200, "OK").await;
        }

        if self.state == ResponseState::Header {
            let _ = self.write("\r\n".as_bytes()).await;
        }

        self.state = ResponseState::Body;

        let data = data.serialize().map_err(|_| Error::SerializeError)?;

        self.write(&data).await.map_err(|_| Error::WriteError)?;

        Ok(())
    }
}

pub trait Serialize {
    fn serialize(self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}
