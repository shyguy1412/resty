use smol::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    HttpMethod, Router, Socket,
    parse::{parse_cookies, parse_url},
    response::ResponseState,
};

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

macro_rules! error {
    ($code: expr, $reason:expr, ($($header:literal => $value: literal),*)) => {
       concat!(
            "HTTP/1.1 ",$code, " ", $reason,"\r\n",
            $($header, ":", $value,"\r\n"),*,
            "\r\n",
        ).as_bytes()
    };
    ($code: expr, $reason:expr) => {
        error!($code, $reason, (
            "Content-Length" => "0",
            "Connection" => "close"
        ))
    }
}

async fn handle_stream<S: Socket + 'static>(mut stream: S::Stream, router: &Router) -> () {
    loop {
        if let Err(body) = handle_request::<S>(&mut stream, router).await {
            let _ = stream.write_all(body).await;
            return;
        };
    }
}

// #[inline(always)]
async fn handle_request<S: Socket + 'static>(
    stream: &mut S::Stream,
    router: &Router,
) -> Result<(), &'static [u8]> {
    let buffer = &mut Vec::with_capacity(4000);

    let Some(header_count) = crate::parse::read_headers::<S>(stream, buffer).await else {
        return Err(error!(400, "Bad Request"));
    };
    buffer.shrink_to_fit();

    let headers = &mut vec![httparse::EMPTY_HEADER; header_count].into_boxed_slice();
    let mut request = httparse::Request::new(headers);

    if request.parse(buffer).is_err() {
        return Err(error!(400, "Bad Request"));
    };

    let Some((handlers, path_params)) = request
        .path
        .zip(request.method.map(Into::<HttpMethod>::into))
        .and_then(|(route, method)| router.handler(route, method))
    else {
        return Err(error!(404, "Not Found"));
    };

    let content_length = request.headers.iter().find_map(|h| {
        match h.name.to_ascii_lowercase() == "content-length" {
            true => u64::from_str_radix(str::from_utf8(h.value).ok()?, 10).ok(),
            false => None,
        }
    });

    let readable = stream.clone();
    let readable = &mut match content_length {
        Some(value) => readable.take(value).boxed_reader(),
        None => readable.boxed_reader(),
    };
    let url = parse_url(&request);
    let cookies = parse_cookies(&request);

    let mut request = crate::Request::new(&request, &url, &cookies, &path_params, readable)
        .await
        .map_err(|_| error!(500, "Internal Server Error"))?;

    let writeable = &mut stream.clone().boxed_writer();
    let mut response = crate::Response::new(writeable, &[]);

    for handler in handlers {
        // let _ = handler(&mut request, &mut response).await;
        if let Err(..) = handler(&mut request, &mut response).await {
            return Err(error!(500, "Internal Server Error"));
        };

        if response.state == ResponseState::Done {
            break;
        }
    }

    response.close().await;

    //fully consume readable before moving on
    if let Some(length) = content_length
        && let Ok(1) = readable.read(&mut [0]).await
    {
        let _ = readable
            .read_to_end(&mut vec![0; length as usize - 1])
            .await;
    };

    Ok(())
}
