use smol::future::FutureExt;
use std::pin::Pin;

// use url::Url;

// use crate::{
//     Result,
//     api::{Endpoint, GET_HELLO_WORLD},
// };

// resty_macros::as_api!(use crate::api);

// pub async fn route(_url: Url) -> Result<Endpoint> {
//     Ok(GET_HELLO_WORLD)
// }

pub struct EndpointFuture {
    inner: Pin<Box<dyn Future<Output = ()>>>,
}

impl EndpointFuture {
    pub fn new<T: FnOnce() -> Pin<Box<dyn Future<Output = ()>>>>(f: T) -> Pin<Box<Self>> {
        Box::pin(EndpointFuture { inner: f() })
    }
}

unsafe impl Send for EndpointFuture {}

impl Future for EndpointFuture {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.inner.poll(cx)
    }
}
