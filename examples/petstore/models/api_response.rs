use resty::schema;
use serde::{Deserialize, Serialize};

#[schema]
#[derive(Deserialize, Serialize)]
pub struct ApiResponse {
    code: i32,
    ty: String,
    message: String,
}
