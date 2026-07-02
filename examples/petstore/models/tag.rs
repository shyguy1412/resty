use resty::schema;
use serde::{Deserialize, Serialize};

#[schema]
#[derive(Deserialize, Serialize)]
pub struct Tag {
    id: i64,
    name: String,
}
