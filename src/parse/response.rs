use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone)]
pub enum Error {
    SerializeError,
    WriteError(&'static str),
    StateError,
    InvalidStatus,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SerializeError => write!(f, "SerializeError"),
            Error::WriteError(e) => write!(f, "WriteError({e})"),
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
    once: bool,
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
            once: false,
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
            .write_all(format!("HTTP/1.1 {status} {reason}\r\n").as_bytes())
            .await;

        self.state = ResponseState::StaticHeaders;

        Box::pin(self.headers(&[])).await?;

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
            if *name == "Connection" && *value == "close" {
                self.once = true;
            }
            let header = format!("{name}: {value}\r\n").into_bytes();
            self.write_all(&header).await.map_err(|e| {
                println!("{e}");
                Error::WriteError("Headers")
            })?;
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

        self.write_all(&format!("Content-Length: {}\r\n\r\n", data.len()).into_bytes())
            .await
            .map_err(|_| Error::WriteError("content length"))?;

        self.write_all(&data)
            .await
            .map_err(|_| Error::WriteError("body"))?;

        let _ = self.flush().await;

        if self.once {
            let _ = self.close().await;
        }

        self.state = ResponseState::Done;

        Ok(())
    }
}

pub trait Serialize {
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

impl<T: Into<Vec<u8>> + Clone> Serialize for T {
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(T::into(self.clone()))
    }
}
