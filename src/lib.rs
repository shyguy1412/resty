//! # Resty
//!
//! Resty is a fast and lightweight framework for concurrent and multithreadable REST APIs.
//!
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
//!
//!     resty::bind(ADDR, &ROUTER);
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
pub use parse::{
    HttpMethod,
    request::{Deserialize, Request},
    response::{Response, Serialize},
};

mod routing;

mod runtime;
pub use runtime::*;

pub use resty_macros::*;
pub use routing::*;

#[doc(hidden)]
pub mod __private {
    pub use linkme;
}
