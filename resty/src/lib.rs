#![doc = include_str!("../README.md")]
pub use resty_macros::*;

mod parse;
pub use parse::HttpMethod;

mod request;
pub use request::Request;
mod response;
pub use response::{Response, RestResponse};
mod serde;
pub use serde::*;

pub mod http_error;

mod routing;
pub use routing::HandlerOrMiddleware::*;
pub use routing::*;

mod runtime;
pub use runtime::*;

mod socket;
pub use socket::*;

#[derive(Debug, Clone)]
pub enum Error {
    SerializeError,
    WriteError(String),
    StateError,
    InvalidStatus,
    MissingContentLength,
    InvalidContentLength,
    ReadError,
    ParseError(String),
    RequestError,
    UnTypedRequest,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingContentLength => write!(f, "MissingContentLength"),
            Error::InvalidContentLength => write!(f, "InvalidContentLength"),
            Error::ReadError => write!(f, "ReadError"),
            Error::ParseError(e) => write!(f, "ParseError({e})"),
            Error::UnTypedRequest => write!(f, "UnTypedRequest"),
            Error::SerializeError => write!(f, "SerializeError"),
            Error::WriteError(e) => write!(f, "WriteError({e})"),
            Error::StateError => write!(f, "StateError"),
            Error::InvalidStatus => write!(f, "InvalidStatus"),
            Error::RequestError => write!(f, "RequestError"),
        }
    }
}

pub type Result = std::result::Result<(), Error>;

#[doc(hidden)]
pub mod __private {
    pub use linkme;

    /// This trait is a marker for Schemas. Do not implement manually, use the Schema derive macro
    pub trait Schema {}
}
