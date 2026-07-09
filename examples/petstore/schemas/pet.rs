use resty::{Response, Schema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Schema, Response)]
#[response(
    Code(200),
    Header("test", "me"),
    ContentType(resty::Json),
    ContentType(resty::XML)
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

impl Pet {
    const CODE: u16 = 200;
    const REASON: &'static str = "OK";
    const HEADERS: &'static [(&'static str, &'static str)] = &[];
}

impl ::resty::RestResponse<resty::Json<Pet>> for Pet {
    const CODE: u16 = Self::CODE;
    const REASON: &'static str = Self::REASON;
    const HEADERS: &'static [(&'static str, &'static str)] = Self::HEADERS;
}

impl ::resty::RestResponse<resty::XML<Pet>> for Pet {
    const CODE: u16 = Self::CODE;
    const REASON: &'static str = Self::REASON;
    const HEADERS: &'static [(&'static str, &'static str)] = Self::HEADERS;
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
