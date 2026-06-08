use std::net::SocketAddrV4;

use smol::{io::AsyncWriteExt, net::TcpListener};

use crate::{Request, Response};

static EXECUTOR: smol::Executor = smol::Executor::new();

#[inline(always)]
pub fn bind(addr: SocketAddrV4) -> () {
    EXECUTOR.spawn(accept_connections(addr)).detach();
}

#[allow(unreachable_code)]
#[inline(always)]
pub fn spawn_thread() -> std::thread::JoinHandle<std::convert::Infallible> {
    std::thread::spawn(|| {
        smol::block_on(async {
            loop {
                EXECUTOR.tick().await;
            }
        })
    })
}

async fn accept_connections(addr: SocketAddrV4) -> () {
    let socket = match TcpListener::bind(addr).await {
        Ok(socket) => socket,
        Err(err) => panic!("{err:?}"),
    };

    loop {
        let Ok((stream, ..)) = socket.accept().await else {
            println!("Dropped incoming connection");
            continue;
        };

        let time = std::time::Instant::now();

        let task = async move {
            let _ = handle_stream(stream).await;
            println!("Request handled in {}µs", time.elapsed().as_micros());
        };

        EXECUTOR.spawn(task).detach();
    }
}

async fn handle_stream(mut stream: smol::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let buffer = &mut Vec::with_capacity(4000);
    let header_count = crate::parse::read_header(&mut stream, buffer).await?;
    buffer.shrink_to_fit();
    let headers = &mut vec![httparse::EMPTY_HEADER; header_count].into_boxed_slice();

    let mut request = httparse::Request::new(headers);

    if let Err(..) = request.parse(buffer) {
        todo!("Handle parsing errors");
    };

    let Some(handler) = request
        .path
        .and_then(|route| crate::routing::ROUTE_TABLE.route(route))
        .zip(request.method.map(Into::into))
        .and_then(|(route, method)| route.endpoints.get(&method))
    else {
        let _ = stream.write("404".as_bytes()).await;
        return Ok(());
    };

    let Some(data) = request
        .method
        .zip(request.path)
        .zip(request.version)
        .map(|((a, b), c)| (a, b, c))
    else {
        todo!("Handle parsing errors");
    };

    handler(
        Request::new(data.0, data.1, data.2, request.headers, stream.clone()),
        Response::new(stream.clone()),
    )
    .await;

    Ok(())
}
