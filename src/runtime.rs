use std::{collections::HashMap, net::SocketAddrV4};

use smol::{io::AsyncWriteExt, net::TcpListener};

static EXECUTOR: smol::Executor = smol::Executor::new();

#[inline(always)]
pub fn bind(addr: SocketAddrV4) -> () {
    EXECUTOR.spawn(accept_connections(addr)).detach();
}

#[inline(always)]
#[doc(hidden)]
pub fn spawn_task<T: Send + 'static>(
    future: impl Future<Output = T> + Send + 'static,
) -> smol::Task<T> {
    EXECUTOR.spawn(future)
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

        spawn_task(metrics(handle_stream(stream))).detach();
    }
}

async fn metrics<T: Future>(task: T) -> T::Output {
    let time = std::time::Instant::now();
    let out = task.await;

    println!("Request handled in {}µs", time.elapsed().as_micros());

    out
}

async fn handle_stream(mut stream: smol::net::TcpStream) -> Result<(), std::io::Error> {
    let buffer = &mut Vec::with_capacity(4000);
    let header_count = crate::parse::read_header(&mut stream, buffer).await?;
    buffer.shrink_to_fit();
    let headers = &mut vec![httparse::EMPTY_HEADER; header_count].into_boxed_slice();

    let mut request = httparse::Request::new(headers);

    if let Err(..) = request.parse(buffer) {
        todo!("Handle parsing errors");
    };

    let headers: HashMap<String, Box<[u8]>> = request.headers.into_iter().fold(
        HashMap::new(),
        |mut map, httparse::Header { name, value }| {
            map.insert(name.to_string(), Box::from(*value));
            map
        },
    );

    println!(
        "Method: {}; Path: {}; HTTP Version: {}",
        request.method.unwrap(),
        request.path.unwrap(),
        request.version.unwrap()
    );

    let Some(handler) = request
        .path
        .and_then(|route| crate::routing::ROUTE_TABLE.route(route))
        .zip(request.method)
        .and_then(|(route, method)| route.endpoints.get(&method.into()))
    else {
        let _ = stream.write("404".as_bytes()).await;
        return Ok(());
    };

    handler(headers, stream).await;

    Ok(())
}
