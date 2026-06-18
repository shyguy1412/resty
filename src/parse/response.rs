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

pub type Writeable = std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send>>;

pub struct Response<'a, Body = (), Error = ()> {
    state: ResponseState,
    writeable: &'a mut Writeable,
    static_headers: &'static [(&'static str, &'static str)],
    once: bool,
    body: PhantomData<Body>,
    error: PhantomData<Error>,
}

impl Deref for Response<'_> {
    type Target = std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send>>;

    fn deref(&self) -> &Self::Target {
        &self.writeable
    }
}

impl DerefMut for Response<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writeable
    }
}

impl<'a, B, E> Response<'a, B, E> {
    pub fn new(
        writeable: &'a mut Writeable,
        static_headers: &'static [(&'static str, &'static str)],
    ) -> Self {
        Self {
            state: ResponseState::Status,
            writeable,
            static_headers,
            once: false,
            body: PhantomData,
            error: PhantomData,
        }
    }

    pub async fn status(&mut self, code: u16, reason: &str) -> Result<(), Error> {
        if code < 100 || code >= 1000 || self.state != ResponseState::Status {
            return Err(Error::InvalidStatus);
        }

        let _ = self
            .writeable
            .write_all(format!("HTTP/1.1 {code} {reason}\r\n").as_bytes())
            .await;

        self.state = ResponseState::StaticHeaders;

        Box::pin(self.headers(&[])).await?;

        Ok(())
    }

    pub async fn headers(&mut self, headers: &[(&str, &str)]) -> Result<(), Error> {
        if self.state >= ResponseState::Body || self.state == ResponseState::Status {
            return Err(Error::StateError);
        }

        if self.state == ResponseState::StaticHeaders {
            self.state = ResponseState::Header;
            Box::pin(self.headers(self.static_headers)).await?;
        }

        for (name, value) in headers {
            if name.to_ascii_lowercase() == "connection" && *value == "close" {
                self.once = true;
            }
            let header = format!("{name}: {value}\r\n").into_bytes();
            self.writeable.write_all(&header).await.map_err(|e| {
                println!("{e}");
                Error::WriteError("Headers")
            })?;
        }

        Ok(())
    }

    /// Requires the Status Line to already be sent
    pub async fn send(&mut self, data: &impl Serialize) -> Result<(), Error> {
        if self.state < ResponseState::Header {
            return Err(Error::StateError);
        }

        let data = data.serialize().map_err(|_| Error::SerializeError)?;

        self.writeable
            .write_all(&format!("Content-Length: {}\r\n\r\n", data.len()).into_bytes())
            .await
            .map_err(|_| Error::WriteError("content length"))?;

        self.writeable
            .write_all(&data)
            .await
            .map_err(|_| Error::WriteError("body"))?;

        let _ = self.writeable.flush().await;

        if self.once {
            let _ = self.writeable.close().await;
        }

        self.state = ResponseState::Done;

        Ok(())
    }
}

impl<B: Serialize, E> Response<'_, B, E> {
    /// Send a response with 200 OK
    pub async fn ok(&mut self, body: &B) -> Result<(), Error> {
        self.status(200, "OK").await?;
        self.send(body).await
    }
}
impl<B, E: Serialize> Response<'_, B, E> {
    //Send an error with a given code and reason
    pub async fn err(&mut self, status: (u16, &str), err: &E) -> Result<(), Error> {
        let (code, reason) = status;
        self.status(code, reason).await?;
        self.send(err).await
    }
}

impl<B, E> Drop for Response<'_, B, E> {
    //ensure the response gets properly terminated
    fn drop(&mut self) {
        if self.state == ResponseState::Done {
            return;
        }
        let _ = smol::block_on(async {
            let _ = self.status(500, "Internal Server Error").await;
            if let Ok(..) = self.headers(&[("Content-Length", "0")]).await {
                let _ = self.writeable.write_all(b"\r\n").await;
            };
        });
    }
}

#[doc = include_str!("../../docs/traits/Serialize.md")]
pub trait Serialize {
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

impl<T: Into<Vec<u8>> + Clone> Serialize for T {
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(T::into(self.clone()))
    }
}
