#[derive(serde::Deserialize)]
pub struct ApiResponse {
    code: i32,
    ty: String,
    message: String,
}
