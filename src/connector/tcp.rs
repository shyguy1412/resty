use std::{marker::PhantomData, net::SocketAddrV4};

use smol::net::{AsyncToSocketAddrs, TcpListener, TcpStream};

pub struct TcpConnector<A = SocketAddrV4>(TcpListener, PhantomData<A>);

impl<A: AsyncToSocketAddrs + Send + Sync> super::Connector for TcpConnector<A> {
    type Error = smol::io::Error;
    type Address = A;
    type Stream = TcpStream;

    async fn bind(addr: Self::Address) -> Result<Box<Self>, Self::Error> {
        TcpListener::bind(addr)
            .await
            .map(|listener| Box::new(TcpConnector(listener, PhantomData)))
    }

    async fn accept(&self) -> Result<Self::Stream, Self::Error> {
        let (stream, ..) = self.0.accept().await?;
        stream.set_nodelay(true)?;

        Ok(stream)
    }
}
