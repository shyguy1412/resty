use resty::schema;
use serde::{Deserialize, Serialize};

#[schema]
#[derive(Deserialize, Serialize)]
pub struct Order {
    #[example(10)]
    id: i64,

    #[example("doggie")]
    pet_id: String,

    #[example(7)]
    quantity: i32,

    #[format("date-time")]
    ship_date: String,

    #[schema(OrderStatus)]
    #[description(Order Status)]
    #[example("approved")]
    status: Status,

    complete: bool,
}

#[schema(Name(OrderStatus))]
#[derive(Deserialize, Serialize)]
enum Status {
    Placed,
    Approved,
    Delivered,
}
