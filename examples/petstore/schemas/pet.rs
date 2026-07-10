use resty::{Response, Schema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Schema, Response)]
#[response(
    Status(200, "OK"),
    ContentType("application/json", resty::Json),
    ContentType("application/xml", resty::XML)
)]
pub struct Pet {
    #[schema(Example(10), Required)]
    id: Option<i64>,

    #[schema(Example("doggie"))]
    name: String,

    category: Option<super::Category>,

    photo_urls: Option<Vec<String>>,
    tags: Option<Vec<super::Tag>>,

    #[schema(Ref(PetStatus))]
    status: Option<Status>,
}

#[derive(Deserialize, Serialize, Schema)]
#[serde(rename_all = "lowercase")]
#[schema(Name(PetStatus))]
enum Status {
    #[schema(Example)]
    Available,
    Pending,
    Sold,
}
