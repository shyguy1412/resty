use resty::Schema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Schema)]
pub struct ApiResponse {
    code: i32,
    ty: String,
    message: String,
}
