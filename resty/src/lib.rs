#![doc = include_str!("../README.md")]

mod parse;
pub use parse::HttpMethod;

mod request;
pub use request::{Deserialize, Request};
mod response;
pub use response::{Response, Serialize};

mod routing;
pub use routing::HandlerOrMiddleware::*;

mod runtime;
pub use runtime::*;

mod socket;
pub use socket::*;

pub use resty_macros::*;
pub use routing::*;

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

    // /// This trait is used as a marker for publicly exported structs
    // /// this way the documentation can validate that all documented structs are known
    // pub trait Public {}

    // macro_rules! public {
    //     ($($ty:ty)*) => ($(impl Public for $ty {})*);
    // }

    // /// A hint to obscure the value
    // pub type Password = String;

    // public! {
    //     String
    //     f32
    //     f64
    //     i32
    //     i64
    // }
}
