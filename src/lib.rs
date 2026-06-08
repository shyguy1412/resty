pub use linkme;
pub use resty_macros::*;
mod api;
// mod router;

use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    net::SocketAddrV4,
    ops::{Deref, DerefMut, Not},
    sync::LazyLock,
    thread::JoinHandle,
    usize,
};

use httparse::{EMPTY_HEADER, Header, Request, parse_headers};

use smol::{
    Executor, Task,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    stream::StreamExt,
};

use url::Url;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

// const ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3333);

#[derive(Clone, Copy, Debug)]
pub enum Error {
    MalformedRequest,
    InvalidMethod,
    NoEndpoint,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

struct Route {
    segments: HashMap<&'static str, Route>,
    endpoints: HashMap<HttpMethod, Handler>,
}

impl Route {
    fn new() -> Self {
        Self {
            segments: HashMap::new(),
            endpoints: HashMap::new(),
        }
    }
}

#[linkme::distributed_slice]
pub static ROUTES: [(&'static [&'static str], Handler, HttpMethod)];

pub type Handler = &'static (dyn Fn(HashMap<String, Box<[u8]>>, TcpStream) -> Task<()> + Sync);

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum HttpMethod {
    GET,
    PUT,
    POST,
    DELETE,
    OPTION,
    TRACE,
    INVALID,
}

static ROUTE_TABLE: LazyLock<Route> = LazyLock::new(build_route_table);

// I have an array somewhat like this ```
// [
//  [a, b, c],
//  [b, b],
//  [a, b, d]
// ]
// ```
// and want to convert it into ```
// [
//  (a, [(b, [(c, []), (d, [])])]),
//  (b, [(b, [])])
// ]

fn build_route_table() -> Route {
    // struct RouteTable(HashMap<&'static str, RouteTable>);

    let mut route_table = Route::new();

    // start with vectors
    //build
    //collapse to boxes

    // let mut route_table = Route::Segment(Box::new([]));

    for (route, handler, method) in ROUTES {
        let mut current_table = &mut route_table;
        for current_segment in *route {
            if !current_table.segments.contains_key(current_segment) {
                current_table.segments.insert(current_segment, Route::new());
            }
            current_table = current_table.segments.get_mut(current_segment).unwrap()
        }
        current_table.endpoints.insert(*method, *handler);
    }

    return route_table;
}

// impl Route {
//     fn segment(&mut self, segment: &str) -> Option<&mut Route> {
//         match self {
//             Route::Segment(items) => items.iter_mut().find_map(|(p, r)| match *p == segment {
//                 true => Some(r),
//                 false => {}
//             }),
//             Route::Endpoint(..) => None,
//         }
//     }
// }

impl From<&str> for HttpMethod {
    fn from(value: &str) -> HttpMethod {
        use HttpMethod::*;
        match value.to_ascii_uppercase().as_str() {
            "GET" => GET,
            "PUT" => PUT,
            "POST" => POST,
            "DELETE" => DELETE,
            "OPTION" => OPTION,
            "TRACE" => TRACE,
            _ => INVALID,
        }
    }
}

// impl std::fmt::Display for HttpMethod {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{self:?}")
//     }
// }

static EXECUTOR: Executor = Executor::new();

#[inline(always)]
pub fn bind(addr: SocketAddrV4) -> () {
    EXECUTOR.spawn(accept_connections(addr)).detach();
}

#[inline(always)]
#[doc(hidden)]
pub fn task(future: impl Future<Output = ()> + Send + 'static) -> Task<()> {
    EXECUTOR.spawn(future)
}

#[allow(unreachable_code)]
#[inline(always)]
pub fn spawn_thread() -> JoinHandle<Infallible> {
    std::thread::spawn(|| smol::block_on(executor_thread()))
}

#[inline(always)]
async fn executor_thread() -> Infallible {
    loop {
        EXECUTOR.tick().await;
    }
}

#[inline(always)]
async fn accept_connections(addr: SocketAddrV4) -> () {
    async fn task_wrapper(mut stream: TcpStream) {
        let time = std::time::Instant::now();
        if let Some(error) = handle_stream(&mut stream)
            .await
            .err()
            .map(|err| format!("{err}"))
        {
            let _ = stream.write(error.as_bytes()).await;
        }
        drop(stream);
        println!("Request handled in {}µs", time.elapsed().as_micros());
    }

    let Ok(socket) = smol::block_on(TcpListener::bind(addr)) else {
        println!("Address already in use");
        std::process::exit(1);
    };
    loop {
        let Ok((stream, ..)) = socket.accept().await else {
            println!("Dropped incoming connection");
            continue;
        };
        EXECUTOR.spawn(task_wrapper(stream)).detach();
    }
}

#[inline(always)]
async fn read_header(stream: &mut TcpStream, buff: &mut Vec<u8>) -> Result<usize> {
    let bytes = &mut stream.bytes();
    let mut header_count = usize::MAX; //adjust for overcounting

    while let Some(byte) = bytes.next().await {
        buff.push(byte?);

        if buff.len() < 4 {
            continue;
        }

        if buff[buff.len() - 2..] == *b"\r\n" {
            header_count = header_count.wrapping_add(1);
        }

        if buff[buff.len() - 4..] == *b"\r\n\r\n" {
            break;
        }
    }
    Ok(header_count)
}

#[inline(always)]
pub async fn handle_stream(stream: &mut smol::net::TcpStream) -> Result<()> {
    let buffer = &mut Vec::with_capacity(4000);
    let header_count = read_header(stream, buffer).await?;
    buffer.shrink_to_fit();
    let headers = &mut vec![EMPTY_HEADER; header_count].into_boxed_slice();

    let request = &mut Request::new(headers);
    request.parse(buffer)?;

    let headers: HashMap<String, Box<[u8]>> =
        request
            .headers
            .into_iter()
            .fold(HashMap::new(), |mut map, Header { name, value }| {
                map.insert(name.to_string(), Box::from(*value));
                map
            });

    println!(
        "Method: {}; Path: {}; HTTP Version: {}",
        request.method.unwrap(),
        request.path.unwrap(),
        request.version.unwrap()
    );

    let handler = request
        .path
        .zip(request.method)
        .ok_or_else(|| Error::MalformedRequest)
        .and_then(|(path, method)| {
            route(path, &ROUTE_TABLE)
                .and_then(|route| route.endpoints.get(&method.into()))
                .ok_or(Error::NoEndpoint)
        })?;

    handler(headers, stream.to_owned()).await;

    Ok(())
}

fn route<'a>(path: &str, from: &'a Route) -> Option<&'a Route> {
    let mut segments = path.strip_prefix("/").unwrap_or(path).split("/");
    let mut route = from;

    while let Some(current_segment) = segments.next() {
        let next = route.segments.get(current_segment);
        match next {
            Some(next_route) => route = next_route,
            None => return None,
        };
    }

    Some(route)
}
