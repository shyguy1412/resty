use smol::io::{AsyncReadExt, AsyncWriteExt};

use crate::{HttpMethod, Router, Socket, parse, routing::HandlerData};

/// Type alias for the Future returned by a Handler
pub type EndpointTask<'a> =
    std::pin::Pin<Box<dyn Future<Output = Result<(), crate::Error>> + 'a + Send>>;

static EXECUTOR: smol::Executor = smol::Executor::new();

#[inline(always)]
pub fn bind<S: Socket + 'static>(
    addr: S::Address,
    router: &'static Router,
) -> Result<(), S::Error> {
    let connector = smol::block_on(S::bind(addr))?;

    EXECUTOR
        .spawn(accept_connections(connector, router))
        .detach();
    Ok(())
}

/// Spawns a worker thread to handle the async task queue.
///
/// At least one worker thread must be spawned
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

async fn accept_connections<S: Socket + 'static>(connector: Box<S>, router: &'static Router) -> () {
    loop {
        let Ok(stream) = connector.accept().await else {
            println!("Dropped incoming connection");
            continue;
        };

        EXECUTOR.spawn(handle_stream::<S>(stream, router)).detach();
    }
}

macro_rules! inline_response {
    ($code: literal $reason:literal on $stream:ident with ($($header:literal => $value: literal)*)) => {
        let _ = $stream.write_all(concat!(
            "HTTP/1.1 ",$code, " ", $reason,"\r\n",
            $($header, ":", $value,"\r\n"),*,
            "\r\n",
        ).as_bytes()).await;
    };
}

async fn handle_stream<S: Socket + 'static>(mut stream: S::Stream, router: &Router) -> () {
    loop {
        // let time = std::time::Instant::now();

        let buffer = &mut Vec::with_capacity(4000);
        let Some(header_count) = crate::parse::read_headers::<S>(&mut stream, buffer).await else {
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
            .zip(request.method.map(Into::<HttpMethod>::into))
            .and_then(|((route, params), method)| Some((route.method(method)?, params)))
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

        if let Err(..) = handler(&mut HandlerData {
            request,
            path_params,
            readable,
            writeable,
        })
        .await
        {
            inline_response!(500 "Internal Server Error" on stream with (
                "Content-Length" => "0"
            ));
            return;
        };

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
