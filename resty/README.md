# resty

resty is a fast and lightweight framework for concurrent and multithreadable REST APIs.

Its capable of manual routing aswell as path based routing similiar to Next.js or Tanstack Router.
It also capable of generating a JSON description of the API that can be used for client snippet and documentation generation.
To do so set `RESTY_DECL_GEN=output.json` as environment variable during compilation.

Known issues with rust-analyzer:

rust-analyzer currently does not support getting the sourcefile path of a Span.
This is vital for path based routing and can be worked around by setting `RESTY_PATH_ROUTING_HINT` to the base directory of your API routes.

Example settings for vscode

```json
{
    "rust-analyzer.server.extraEnv": {
        "RESTY_PATH_ROUTING_HINT": "src/routes/"
    }
}
```

## Example

```rust
#[resty::router]
static ROUTER: LazyLock<Router>;

#[resty::endpoint(
    Route("/"),
    Router(ROUTER),
    Method(GET),
    Header("Content-Type", "text/html; charset=utf-8")
)]
async fn get_hello_world<'a>(_req: &mut Request<'a>, res: &mut Response<'a>) -> resty::Result {
    res.ok(&"Hello World!").await?;

    Ok(())
}

fn main() -> ExitCode {
    const ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3333);

    if let Err(error) = resty::bind::<TcpSocket>(ADDR, &ROUTER) {
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

    std::thread::park();

    return ExitCode::SUCCESS;
}
```
