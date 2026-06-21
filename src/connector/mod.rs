use smol::io::{AsyncRead, AsyncWrite};

mod tcp;
pub use tcp::TcpConnector;

mod unix;

pub trait Connector: Send {
    type Error;
    type Address;
    type Stream: AsyncRead + AsyncWrite + Unpin + Clone + Send + 'static;

    fn bind(
        addr: Self::Address,
    ) -> impl std::future::Future<Output = Result<Box<Self>, Self::Error>>;

    fn accept(&self)
    -> impl std::future::Future<Output = Result<Self::Stream, Self::Error>> + Send;
}
