use std::net::SocketAddrV4;

use smol::{io::AsyncWriteExt, net::TcpListener};

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
            println!("Request handled in {}ms", time.elapsed().as_millis());
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

    handler(request, stream).await;

    Ok(())
}
