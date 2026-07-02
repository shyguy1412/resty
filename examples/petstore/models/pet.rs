use resty::Schema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Schema)]
pub struct Pet {
    #[schema(Example(10))]
    id: i64,

    #[schema(Example("doggie"))]
    name: String,

    category: super::Category,
    photo_urls: Vec<String>,
    tags: super::Tag,

    #[schema(PetStatus)]
    status: Status,
}

#[derive(Deserialize, Serialize, Schema)]
#[schema(Name(PetStatus))]
enum Status {
    #[schema(Example)]
    Available,
    Pending,
    Sold,
}
