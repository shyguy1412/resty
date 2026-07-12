use smol::io::AsyncWriteExt;

use crate::{AsyncIterator, ContentType, Error, RestResponse, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ResponseState {
    Status,
    StaticHeaders,
    Header,
    Body,
    Done,
    RawAccess,
}

pub type Writeable = std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send>>;

/// An HTTP response. Provides a set of high level APIs to send or stream reponse data.
pub struct Response<'a> {
    pub(crate) state: ResponseState,
    writeable: &'a mut Writeable,
    static_headers: &'static [(&'static str, &'static str)],
    once: bool,
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
                .map_err(|e| Error::WriteError(e.to_string()))?;
        }

        Ok(())
    }

    ///send arbitrary serializable data
    pub async fn send(&mut self, data: &impl Serialize) -> Result<(), Error> {
        if self.state != ResponseState::Header {
            return Err(Error::StateError);
        }

        let data = Serialize::serialize(data).map_err(|_| Error::SerializeError)?;

        self.writeable
            .write_all(&format!("Content-Length: {}\r\n\r\n", data.len()).into_bytes())
            .await
            .map_err(|e| e.to_string())
            .map_err(Error::WriteError)?;

        self.writeable
            .write_all(&data)
            .await
            .map_err(|e| e.to_string())
            .map_err(Error::WriteError)?;

        let _ = self.writeable.flush().await;

        if self.once {
            let _ = self.writeable.close().await;
        }

        self.state = ResponseState::Done;

        Ok(())
    }

    ///Send a response as defined by the openapi rest spec
    pub async fn respond<C: ContentType<R>, R: RestResponse>(
        &mut self,
        response: C,
    ) -> Result<(), Error> {
        self.status(R::CODE, R::REASON).await?;
        self.headers(R::HEADERS).await?;
        self.headers(&[("Content-Type", C::CONTENT_TYPE)]).await?;
        self.send(&response).await?;
        Ok(())
    }

    /// Stream elements of an async iterator
    /// currently only supported in combination with Server Sent Events
    pub async fn stream<E: Serialize, D: std::ops::Deref<Target = E>>(
        &mut self,
        mut stream: impl AsyncIterator<Item = D>,
    ) -> Result<(), Error> {
        self.status(200, "OK").await?;
        self.headers(&[("Content-Type", "text/event-stream")])
            .await?;
        self.writeable
            .write_all(b"\r\n\r\n")
            .await
            .map_err(|e| e.to_string())
            .map_err(Error::WriteError)?;

        while let Some(data) = stream.next().await {
            let data = Serialize::serialize(data.deref()).map_err(|_| Error::SerializeError)?;
            self.writeable
                .write_all(&data)
                .await
                .map_err(|e| e.to_string())
                .map_err(Error::WriteError)?;
        }

        Ok(())
    }

    /// Close the response.
    /// This does not gurantee that the underlying stream will be closed aswell
    pub async fn close(&mut self) {
        if self.state == ResponseState::Done {
            return;
        }
        if self.state == ResponseState::RawAccess {
            let _ = self.writeable.close().await;
        }
        let _ = self.status(500, "Internal Server Error").await;
        let _ = self.send(&"").await;
    }

    /// Provides access to the raw byte stream of a response.
    /// Calling this function will cause future calls to high level APIs to return a state error
    /// Additionally, this will cause the connection to be forcefully closed after the request is done
    pub fn raw_stream<'s>(&'s mut self) -> &'s mut Writeable {
        self.state = ResponseState::RawAccess;
        &mut self.writeable
    }
}
