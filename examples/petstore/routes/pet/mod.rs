use resty::{AsyncWriteExt, Json, Request, Response, XML, endpoint, http_error};

use crate::schemas::Pet;

#[endpoint(
    Tag("pet"),
    Summary("Update an existing pet"),
    Description("Update an existing bet by Id"),
    Request(
        Description("Update an existent pet in the store"),
        Schema("application/json", Pet),
        Schema("application/xml", Pet),
        Schema("application/x-www-form-urlencoded", Pet),
        Required
    ),
    Response(200, Pet),
    Response(400, "Invalid ID supplied"),
    Response(404, "Pet not found"),
    Response(422, "Validation exception"),
    Security(Name("petstore_auth"), Scope("write:pets"), Scope("read:pets")),
    Method(PUT)
)]
async fn put_pet<'a, 'b>(req: &mut Request<'a, 'b>, res: &mut Response<'a>) -> resty::Result {
    let Json(body): Json<Pet> = req.body().await?;

    Ok(())
}
