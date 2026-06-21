use smol::io::{AsyncRead, AsyncWrite};

mod tcp;
pub use tcp::TcpScocket;

mod unix;
pub use unix::UnixSocket;

/// The Socket trait allows for implementation of custom transport layers
pub trait Socket: Send {
    type Error;
    type Address;
    type Stream: AsyncRead + AsyncWrite + Unpin + Clone + Send;

    fn bind(
        addr: Self::Address,
    ) -> impl std::future::Future<Output = Result<Box<Self>, Self::Error>>;

    fn accept(&self)
    -> impl std::future::Future<Output = Result<Self::Stream, Self::Error>> + Send;
}
