# Macro to register new endpoint handler

## Usage

This macro supports the following arguments:

- Router: The Router that this endpoint should be added to. This is inferred with path routing
- Route: The endpoing path for that handler. This is infered with path routing
- Method: Comma separated list of HTTP methods this handler should handle
- Header: Repeatable. Declares a Header that should always be sent

```rust
#[resty::endpoint(
    Router(ROUTER),
    Route("/"),
    Method(GET),
    Header("Content-Type", "text/html; charset=utf-8")
)]
async fn get_hello_world<'a>(_req: &mut Request<'a>, res: &mut Response<'a, &'static str>) {
    let _ = res.ok(&"Hello World!").await;
}
```
