use resty::Schema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Schema)]
#[schema(Description("Some Order"))]
pub struct Order {
    #[schema(Example(10))]
    id: i64,

    #[schema(Example("doggie"))]
    pet_id: String,

    #[schema(Example(7))]
    quantity: i32,

    #[schema(Format("date-time"))]
    ship_date: String,

    #[schema(Ref(OrderStatus), Description("Order Status"), Example("approved"))]
    status: Status,

    complete: bool,
}

#[derive(Deserialize, Serialize, Schema)]
#[schema(Name(OrderStatus), Type("string"))]
enum Status {
    #[schema(Repr(placed))]
    OrderPlaced,
    #[schema(Example)]
    Approved,
    Delivered,
}
