use std::net::SocketAddrV4;

use smol::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use crate::{Router, routing::HandlerData};

/// Type alias for the Future returned by a Handler
pub type EndpointTask<'a> = std::pin::Pin<Box<dyn Future<Output = ()> + 'a + Send>>;

static EXECUTOR: smol::Executor = smol::Executor::new();

#[inline(always)]
pub fn bind(addr: SocketAddrV4, router: &'static Router) -> () {
    EXECUTOR.spawn(accept_connections(addr, router)).detach();
}

/// Spawns a worker thread to handle the async task queue.
///
/// At least one worker thread must be spawned
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

async fn accept_connections(addr: SocketAddrV4, router: &'static Router) -> () {
    let socket = match TcpListener::bind(addr).await {
        Ok(socket) => socket,
        Err(err) => panic!("{err:?}"),
    };

    loop {
        let Ok((stream, ..)) = socket.accept().await else {
            println!("Dropped incoming connection");
            continue;
        };

        if let Err(..) = stream.set_nodelay(true) {
            continue;
        };

        EXECUTOR.spawn(handle_stream(stream, router)).detach();
    }
}

macro_rules! inline_response {
    ($code: literal $reason:literal on $stream:ident with ($($header:literal => $value: literal)*)) => {
        let _ = $stream.write_all(concat!(
            "HTTP/1.1 ",$code, " ", $reason,"\r\n",
            $($header, ":", $value,"\r\n"),*,
            "\r\n",
        ).as_bytes());
    };
}

async fn handle_stream(mut stream: smol::net::TcpStream, router: &Router) -> () {
    loop {
        // let time = std::time::Instant::now();

        let buffer = &mut Vec::with_capacity(4000);
        let Some(header_count) = crate::parse::read_headers(&mut stream, buffer).await else {
            return;
        };
        buffer.shrink_to_fit();

        let headers = &mut vec![httparse::EMPTY_HEADER; header_count].into_boxed_slice();
        let mut request = httparse::Request::new(headers);

        if let Err(..) = request.parse(buffer) {
            inline_response!(403 "Malformed Request Headers" on stream with (
                "Content-Length" => "0"
            ));
            return;
        };

        let Some((handler, path_params)) = request
            .path
            .and_then(|route| router.route(route))
            .zip(request.method.map(Into::into))
            .and_then(|((route, params), method)| Some((route.endpoints.get(&method)?, params)))
        // .or_else(|| FALLBACK.get(0).map(|fallback| (fallback, vec![])))
        else {
            inline_response!(404 "Not Found" on stream with (
                "Content-Length" => "0"
            ));
            return;
        };

        let content_length = request.headers.iter().find_map(|h| {
            match h.name.to_ascii_lowercase() == "content-length" {
                true => u64::from_str_radix(str::from_utf8(h.value).ok()?, 10).ok(),
                false => None,
            }
        });

        let writeable = &mut stream.clone().boxed_writer();

        let readable = &mut match content_length {
            Some(value) => stream.clone().take(value).boxed_reader(),
            None => stream.clone().boxed_reader(),
        };
        handler(&mut HandlerData {
            request,
            path_params,
            readable,
            writeable,
        })
        .await;

        //fully consume readable before moving on
        if let Some(length) = content_length
            && let Ok(1) = readable.read(&mut [0]).await
        {
            let _ = readable
                .read_to_end(&mut vec![0; length as usize - 1])
                .await;
        }

        // println!("Request handled in {}µs", time.elapsed().as_micros());
    }
}
