use std::ops::{Deref, DerefMut};

use smol::io::AsyncWriteExt;

use crate::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ResponseState {
    Status,
    StaticHeaders,
    Header,
    Body,
    Done,
}

pub type Writeable = std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send>>;

pub struct Response<'a> {
    pub(crate) state: ResponseState,
    writeable: &'a mut Writeable,
    static_headers: &'static [(&'static str, &'static str)],
    once: bool,
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

impl<'a> Response<'a> {
    pub fn new(
        writeable: &'a mut Writeable,
        static_headers: &'static [(&'static str, &'static str)],
    ) -> Self {
        Self {
            state: ResponseState::Status,
            writeable,
            static_headers,
            once: false,
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
            self.writeable
                .write_all(&header)
                .await
                .map_err(|e| Error::WriteError(e))?;
        }

        Ok(())
    }

    /// Requires the Status Line to already be sent
    /// Prefer using the `ok` and `err` methods
    pub async fn send(&mut self, data: &impl Serialize) -> Result<(), Error> {
        if self.state != ResponseState::Header {
            return Err(Error::StateError);
        }

        let data = data.serialize().map_err(|_| Error::SerializeError)?;

        self.writeable
            .write_all(&format!("Content-Length: {}\r\n\r\n", data.len()).into_bytes())
            .await
            .map_err(Error::WriteError)?;

        self.writeable
            .write_all(&data)
            .await
            .map_err(Error::WriteError)?;

        let _ = self.writeable.flush().await;

        if self.once {
            let _ = self.writeable.close().await;
        }

        self.state = ResponseState::Done;

        Ok(())
    }

    pub async fn ok(&mut self, body: &impl Serialize) -> Result<(), Error> {
        self.status(200, "OK").await?;
        self.send(body).await
    }

    pub async fn err(&mut self, status: (u16, &str), err: &impl Serialize) -> Result<(), Error> {
        let (code, reason) = status;
        self.status(code, reason).await?;
        self.send(err).await
    }

    pub async fn close(&mut self) {
        if self.state == ResponseState::Done {
            return;
        }
        let _ = self.status(500, "Internal Server Error").await;
        let _ = self.send(&"").await;
    }
}

#[doc = include_str!("../docs/traits/Serialize.md")]
pub trait Serialize {
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

impl<T: Into<Vec<u8>> + Clone> Serialize for T {
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(T::into(self.clone()))
    }
}
