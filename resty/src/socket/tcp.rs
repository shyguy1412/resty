use std::{marker::PhantomData, net::SocketAddrV4};

use smol::net::{AsyncToSocketAddrs, TcpListener, TcpStream};

/// Implementation for a TCP transport layer
pub struct TcpScocket<A = SocketAddrV4>(TcpListener, PhantomData<A>);

impl<A: AsyncToSocketAddrs + Send + Sync> super::Socket for TcpScocket<A> {
    type Error = smol::io::Error;
    type Address = A;
    type Stream = TcpStream;

    async fn bind(addr: Self::Address) -> Result<Box<Self>, Self::Error> {
        TcpListener::bind(addr)
            .await
            .map(|listener| Box::new(TcpScocket(listener, PhantomData)))
    }

    async fn accept(&self) -> Result<Self::Stream, Self::Error> {
        let (stream, ..) = self.0.accept().await?;
        stream.set_nodelay(true)?;

        Ok(stream)
    }
}
