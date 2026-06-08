use std::collections::HashMap;

use smol::Task;
use smol::{io::AsyncWriteExt, net::TcpStream};

// pub type Endpoint = &'static dyn Fn(OwnedRequest, TcpStream) -> Pin<Box<()>>;
// use crate::EXECUTOR;
pub fn get_hello_world(headers: HashMap<String, Box<[u8]>>, stream: TcpStream) -> Task<()> {
    pub async fn get_hello_world_impl(
        mut _headers: HashMap<String, Box<[u8]>>,
        mut stream: TcpStream,
    ) {
        if let Err(err) = stream.write_all(b"Hello World!").await {
            println!("{err}");
        };
    }
    crate::task(get_hello_world_impl(headers, stream))
    // EXECUTOR.spawn(get_hello_world_impl(headers, stream))
}

// pub const GET_HELLO_WORLD: Endpoint = &|request: OwnedRequest, stream: TcpStream| {
//     EndpointFuture::new(move || Box::pin(get_hello_world(request, stream)))
// };
