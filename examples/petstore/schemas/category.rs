use resty::Schema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Schema)]
pub struct Category {
    #[schema(Example(1))]
    id: i64,
    #[schema(Example("Dogs"))]
    name: String,
}
