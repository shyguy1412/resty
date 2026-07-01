# Serialize

The resty::Serialize trait provides a unified interface for serialization to a Vec\<u8\>.
This allows free choice of serialization format and framework.

## Usage

An example using serde and serde_json

```rust
#[derive(serde::Serialize, resty::Serialize)]
#[serializer(crate::serialize)]
struct MyResponse {
  foo: String,
  bar: f64
};


fn serialize<'a, T: serde::Serialize<'a>>(
    data: &'a [u8],
) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::to_vec(&data)?)
}
```
