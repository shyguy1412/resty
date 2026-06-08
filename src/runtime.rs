use std::{collections::HashMap, net::SocketAddrV4, string::FromUtf8Error};

use smol::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

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

        spawn_task(swallow(handle_stream(stream))).detach();
    }
}

async fn swallow<T: Future>(task: T) -> () {
    let time = std::time::Instant::now();
    task.await;

    println!("Request handled in {}µs", time.elapsed().as_micros());
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

    let headers = request.headers.into_iter().try_fold(
        HashMap::<String, Vec<String>>::new(),
        |mut map, header| -> Result<_, FromUtf8Error> {
            let value = String::from_utf8(header.value.to_vec())?;
            match map.get_mut(header.name) {
                Some(vec) => vec.push(value),
                None => {
                    map.insert(header.name.to_string(), vec![value]);
                }
            };

            Ok(map)
        },
    )?;

    // println!(
    //     "Method: {}; Path: {}; HTTP Version: {}",
    //     request.method.unwrap(),
    //     request.path.unwrap(),
    //     request.version.unwrap()
    // );

    let Some((handler, method)) = request
        .path
        .and_then(|route| crate::routing::ROUTE_TABLE.route(route))
        .zip(request.method.map(Into::into))
        .and_then(|(route, method)| Some((route.endpoints.get(&method)?, method)))
    else {
        let _ = stream.write("404".as_bytes()).await;
        return Ok(());
    };

    handler(
        crate::Request {
            method,
            headers,
            body: (),
            readable: stream.clone().boxed_reader(),
        },
        crate::Response {
            writeable: stream.clone().boxed_writer(),
        },
    )
    .await;

    Ok(())
}
