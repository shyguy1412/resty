# Macro to register new middleware

Middlewares work very similiar to endpoints, except that they act on any sub route. This means a middleware registered for `/` acts on every single request. There can only be one middleware for each base path. Middlewares are run in the order they are encountered in during routing. This means with middlewares set for

- `/`
- `/home/user`
- `/home`

The middlewares are guranteed to run in the order

1. `/`
1. `/home`
1. `/home/user`

## Usage

This macro supports the following arguments:

- Router: The Router that this endpoint should be added to. This is inferred with path routing
- Route: The endpoing path for that handler. This is infered with path routing

```rust
#[resty::middleware(
    Router(ROUTER),
    Route("/"),
)]
async fn auth<'a>(req: &mut Request<'a>, res: &mut Response<'a, &'static str>) {
    if !authenticate(req) {
        res.status(403, "Not Authorized").await?;
        res.close().await?;
    }

    Ok(())
}
```
