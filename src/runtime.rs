use std::net::SocketAddrV4;

use smol::{io::AsyncWriteExt, net::TcpListener};

use crate::routing::{FALLBACK, HandlerData};

pub type EndpointTask<'a> = std::pin::Pin<Box<dyn Future<Output = ()> + 'a + Send>>;

static EXECUTOR: smol::Executor = smol::Executor::new();

#[inline(always)]
pub fn bind(addr: SocketAddrV4) -> () {
    EXECUTOR.spawn(accept_connections(addr)).detach();
}

#[allow(unreachable_code)]
#[inline(always)]
pub fn spawn_thread() -> std::thread::JoinHandle<std::convert::Infallible> {
    println!("Thread Spawned");
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

        println!("Connection accepted");

        let time = std::time::Instant::now();

        let task = async move {
            let _ = handle_stream(stream).await;
            println!("Request handled in {}µs", time.elapsed().as_micros());
            // println!("Request handled in {}ms", time.elapsed().as_millis());
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
        let _ = stream
            .write(b"HTTP/1.1 403 Malformed Request Headers\r\n")
            .await;
        return Ok(());
    };

    let Some((handler, path_params)) = request
        .path
        .and_then(|route| crate::routing::ROUTE_TABLE.route(route))
        .zip(request.method.map(Into::into))
        .and_then(|((route, params), method)| Some((route.endpoints.get(&method)?, params)))
        .or_else(|| FALLBACK.get(0).map(|fallback| (fallback, vec![])))
    else {
        let _ = stream.write(b"HTTP/1.1 404 Not Found\r\n").await;
        return Ok(());
    };

    handler(&mut HandlerData {
        request,
        path_params,
        stream,
    })
    .await;

    Ok(())
}
