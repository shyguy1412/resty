use resty::Schema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Schema)]
pub struct User {
    #[schema(Example(10))]
    id: i64,

    #[schema(Example("theUser"))]
    username: String,

    #[schema(Example("John"))]
    first_name: String,

    #[schema(Example("James"))]
    last_name: String,

    #[schema(Example("john@email.com"))]
    email: String,

    #[schema(Example("12345"))]
    password: String,

    #[schema(Example("12345"))]
    phone: String,

    #[schema(Example(1), Description("User Status"))]
    user_status: i32,
}
