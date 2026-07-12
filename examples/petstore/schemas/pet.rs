use resty::{Response, Schema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Schema, Response)]
#[response(
    Description("Operation Successfull"),
    Status(200, "OK"),
    ContentType("application/json"),
    ContentType("application/xml")
)]
pub struct Pet {
    #[schema(Example(10), Required)]
    pub id: Option<i64>,

    #[schema(Example("doggie"))]
    pub name: String,

    pub category: Option<super::Category>,

    pub photo_urls: Option<Vec<String>>,
    pub tags: Option<Vec<super::Tag>>,

    #[schema(Ref(PetStatus))]
    pub status: Option<Status>,
}

#[derive(Deserialize, Serialize, Schema)]
#[serde(rename_all = "lowercase")]
#[schema(Name(PetStatus))]
pub enum Status {
    #[schema(Example)]
    Available,
    Pending,
    Sold,
}
