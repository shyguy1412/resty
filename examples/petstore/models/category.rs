use resty::schema;
use serde::{Deserialize, Serialize};

#[schema]
#[derive(Deserialize, Serialize)]
pub struct Category {
    #[example(1)]
    id: i64,
    #[example("Dogs")]
    name: String,
}
