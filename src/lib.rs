//! # resty
//!
//! resty is a fast and lightweight framework for concurrent and multithreadable REST APIs.
//!
//! Its capable of manual routing aswell as path based routing similiar to Next.js or Tanstack Router.
//! It also capable of generating a JSON description of the API that can be used for client snippet and documentation generation.
//! To do so set `RESTY_DECL_GEN=output.json` as environment variable during compilation.
//!
//! Known issues with rust-analyzer:
//!
//! rust-analyzer currently does not support getting the sourcefile path of a Span.
//! This is vital for path based routing and can be worked around by setting `RESTY_PATH_ROUTING_HINT` to the base directory of your API routes.
//!
//! Example settings for vscode
//! ```
//! {
//!     "rust-analyzer.server.extraEnv": {
//!        "RESTY_PATH_ROUTING_HINT": "src/routes/"
//!     }
//! }
//! ```
//!
//! ## Example
//!
//! ```rust
//! #[resty::use_manual_routing]
//! static ROUTER: LazyLock<Router>;
//!
//! #[resty::endpoint(
//!     Router(ROUTER),
//!     Path("/"),
//!     Method(GET),
//!     Header("Content-Type", "text/html; charset=utf-8")
//! )]
//! async fn get_hello_world<'a>(_req: &mut Request<'a>, res: &mut Response<'a, &'static str>) {
//!     let _ = res.ok(&"Hello World!").await;
//! }
//!
//! fn main() -> ExitCode {
//!     const ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3333);
//!     if let Err(error) = resty::bind::<TcpSocket>(ADDR, &ROUTER) {
//!         println!("{error:?}");
//!         return ExitCode::FAILURE;
//!     }
//!     
//!     println!("Listening on port 3333");
//!
//!     let _: Vec<_> = std::thread::available_parallelism()
//!         .ok()
//!         .map(|n| 0..n.get())
//!         .unwrap_or(0..1)
//!         .map(|_| resty::spawn_thread())
//!         .collect();
//!
//!     std::thread::park();
//!
//!     return ExitCode::SUCCESS;
//! }
//! ```

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

    /// This trait is used as a marker for publicly exported structs
    /// this way the documentation can validate that all documented structs are known
    pub trait Public {}
}
