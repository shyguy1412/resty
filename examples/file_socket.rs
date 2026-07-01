//! This example implements a custom Socket that reads HTTP Requests from a file

use std::{
    pin::Pin,
    process::ExitCode,
    sync::LazyLock,
    thread,
    time::{Duration, SystemTime},
};

use smol::{
    fs::{File, OpenOptions},
    io::{AsyncRead, AsyncWrite},
};

use resty::{Request, Response, Router, endpoint, router};

pub struct FileSocket(&'static str);

impl resty::Socket for FileSocket {
    type Error = std::io::Error;
    type Address = &'static str;
    type Stream = FileStream;

    async fn bind(addr: Self::Address) -> Result<Box<Self>, Self::Error> {
        Ok(Box::new(FileSocket(addr)))
    }

    async fn accept(&self) -> Result<Self::Stream, Self::Error> {
        let instant = SystemTime::now();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.0)
            .await?;

        loop {
            let meta = file.metadata().await?;

            let mtime = meta.modified()?;

            let Ok(..) = mtime.duration_since(instant) else {
                continue;
            };

            break;
        }

        thread::sleep(Duration::from_millis(100));

        Ok(FileStream(self.0, file))
    }
}
pub struct FileStream(&'static str, File);

impl Clone for FileStream {
    fn clone(&self) -> Self {
        let file = smol::block_on(async {
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(self.0)
                .await
                .unwrap()
        });

        FileStream(self.0, file)
    }
}

impl AsyncRead for FileStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        AsyncRead::poll_read(Pin::new(&mut self.1), cx, buf)
    }
}

impl AsyncWrite for FileStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        AsyncWrite::poll_write(Pin::new(&mut self.1), cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.1), cx)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        AsyncWrite::poll_close(Pin::new(&mut self.1), cx)
    }
}

#[router]
static ROUTER: LazyLock<Router>;

#[endpoint(Method(GET), Route("/"), Router(ROUTER))]
async fn get_hello_world<'a>(_req: &mut Request<'a>, res: &mut Response<'a>) -> resty::Result {
    res.ok(&"Hello World!").await?;

    Ok(())
}

fn main() -> ExitCode {
    println!("{}", *ROUTER);

    let _ = std::fs::remove_file("./file-socket");
    let _ = std::fs::write("./file-socket", &[]);
    if let Err(error) = resty::bind::<FileSocket>("./file-socket", &ROUTER) {
        println!("{error:?}");
        return ExitCode::FAILURE;
    }

    println!("Listening on port 3333");

    let _: Vec<_> = std::thread::available_parallelism()
        .ok()
        .map(|n| 0..n.get())
        .unwrap_or(0..1)
        .map(|_| resty::spawn_thread())
        .collect();

    thread::park();

    return ExitCode::SUCCESS;
}
