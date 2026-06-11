use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Copy)]
pub enum Error {
    SerializeError,
    WriteError,
    StateError,
    InvalidStatus,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SerializeError => write!(f, "SerializeError"),
            Error::WriteError => write!(f, "WriteError"),
            Error::StateError => write!(f, "StateError"),
            Error::InvalidStatus => write!(f, "InvalidStatus"),
        }
    }
}

use smol::io::AsyncWriteExt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ResponseState {
    Status,
    StaticHeaders,
    Header,
    Body,
    Done,
}

pub struct Response<B = ()> {
    state: ResponseState,
    status: Option<u16>,
    writeable: std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send>>,
    static_headers: &'static [(&'static str, &'static str)],
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
    pub fn new(
        stream: smol::net::TcpStream,
        static_headers: &'static [(&'static str, &'static str)],
    ) -> Self {
        Self {
            state: ResponseState::Status,
            status: None,
            writeable: stream.boxed_writer(),
            static_headers,
            body: PhantomData,
        }
    }

    pub async fn status(&mut self, status: u16, reason: &str) -> Result<u16, Error> {
        if let Some(status) = self.status {
            return Ok(status);
        }

        if status < 100 || status >= 1000 || self.state != ResponseState::Status {
            return Err(Error::InvalidStatus);
        }

        self.status.replace(status);

        let _ = self
            .write(format!("HTTP/1.1 {status} {reason}\r\n").as_bytes())
            .await;

        Box::pin(self.headers(&[])).await?;

        self.state = ResponseState::StaticHeaders;

        Ok(status)
    }

    pub async fn headers(&mut self, headers: &[(&str, &str)]) -> Result<(), Error> {
        if self.state == ResponseState::Body {
            return Err(Error::StateError);
        }

        if self.state == ResponseState::Status {
            self.status(200, "OK").await?;
            self.state = ResponseState::StaticHeaders;
        };

        if self.state == ResponseState::StaticHeaders {
            self.state = ResponseState::Header;
            Box::pin(self.headers(self.static_headers)).await?;
        }

        for (name, value) in headers {
            self.write(name.as_bytes())
                .await
                .map_err(|_| Error::WriteError)?;
            self.write(b": ").await.map_err(|_| Error::WriteError)?;
            self.write(value.as_bytes())
                .await
                .map_err(|_| Error::WriteError)?;
            self.write(b"\r\n").await.map_err(|_| Error::WriteError)?;
        }

        Ok(())
    }
}

impl<B: Serialize> Response<B> {
    pub async fn send(&mut self, data: &B) -> Result<(), Error> {
        if self.state == ResponseState::Done {
            return Err(Error::StateError);
        }

        if self.state == ResponseState::Status {
            self.status(200, "OK").await?;
        }

        if self.state < ResponseState::Body {
            self.headers(&[]).await?;
        }

        let data = data.serialize().map_err(|_| Error::SerializeError)?;

        self.write(b"\r\n").await.map_err(|_| Error::WriteError)?;
        self.write(&data).await.map_err(|_| Error::WriteError)?;

        self.state = ResponseState::Done;

        Ok(())
    }
}

pub trait Serialize {
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}
