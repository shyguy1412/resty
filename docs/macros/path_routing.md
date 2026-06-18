# Macro to automaticall route via file path

Using path routing allows you to omit the `Path` and `Route` arguments for the `endpoint` macro.

## Usage

```rust
#[resty::use_path_routing("./api")]
static ROUTER: LazyLock<Router>;
```

This will add all rust sourcefiles in the `api` foulder and use their path relative to `./api` as API route.

This means that a structure like:

```txt
- main.rs (declares ROUTER)
  - api
    - %404.rs
    - v0
      - [user].rs
      - mod.rs
```

with each file having at least one `endpoint` macro invocation would register

- a 404 fallback handler
- `/v0/[user]` where `[user]` is a path parameter
- `/v0`

to ROUTER
