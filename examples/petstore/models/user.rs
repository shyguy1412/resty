use resty::schema;
use serde::{Deserialize, Serialize};

#[schema]
#[derive(Deserialize, Serialize)]
pub struct User {
    #[example(10)]
    id: i64,

    #[example("theUser")]
    username: String,

    #[example("John")]
    first_name: String,

    #[example("James")]
    last_name: String,

    #[example("john@email.com")]
    email: String,

    #[example("12345")]
    password: String,

    #[example("12345")]
    phone: String,

    #[example(1)]
    #[description("User Status")]
    user_status: i32,
}
