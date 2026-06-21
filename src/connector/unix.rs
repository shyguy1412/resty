use std::{marker::PhantomData, path::Path};

use smol::net::unix::{UnixListener, UnixStream};

pub struct UnixConnector<P = &'static str>(UnixListener, PhantomData<P>);

impl<P: AsRef<Path> + Send + Sync> super::Connector for UnixConnector<P> {
    type Error = smol::io::Error;
    type Address = P;
    type Stream = UnixStream;

    async fn bind(addr: Self::Address) -> Result<Box<Self>, Self::Error> {
        UnixListener::bind(addr).map(|listener| Box::new(UnixConnector(listener, PhantomData)))
    }

    async fn accept(&self) -> Result<Self::Stream, Self::Error> {
        let (stream, ..) = self.0.accept().await?;
        Ok(stream)
    }
}
