use resty::schema;
use serde::{Deserialize, Serialize};

#[schema]
#[derive(Deserialize, Serialize)]
pub struct User {
    #[example(10)]
    id: i64,

    #[example("doggie")]
    name: String,

    category: super::Category,
    photo_urls: Vec<String>,
    tags: super::Tag,

    #[schema(PetStatus)]
    status: Status,
}

#[schema(Name(PetStatus))]
#[derive(Deserialize, Serialize)]
enum Status {
    Available,
    Pending,
    Sold,
}
