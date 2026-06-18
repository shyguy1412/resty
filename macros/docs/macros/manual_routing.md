# Macro to manually route endpoint handlers

## Usage

```rust
#[resty::use_manual_routing]
static ROUTER: LazyLock<Router>;

#[resty::endpoint(
    Router(ROUTER),
    Path("/"),
    Method(GET),
    Header("Content-Type", "text/html; charset=utf-8")
)]
async fn get_hello_world<'a>(_req: &mut Request<'a>, res: &mut Response<'a, &'static str>) {
    let _ = res.ok(&"Hello World!").await;
}
```
