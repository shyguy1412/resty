# Deserialize

The resty::Deserialize trait provides a unified interface for deserialization from a Vec\<u8\>.
This allows free choice of serialization format and framework.

## Usage

An example using serde and serde_json

```rs
#[derive(serde::Deserialize, resty::Deserialize)]
#[deserializer(crate::deserialize)]
struct MyResponse {
  foo: String,
  bar: f64
};


fn deserialize<'a, T: serde::Deserialize<'a>>(
    data: &'a [u8],
) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(data).inspect_err(|e| println!("{e:?}"))?)
}
```
