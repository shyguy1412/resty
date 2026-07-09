use resty::Schema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Schema)]
pub struct Tag {
    id: i64,
    name: String,
}
