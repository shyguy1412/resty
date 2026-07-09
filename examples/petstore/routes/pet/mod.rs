use resty::{Json, Request, Response, XML, endpoint};

use crate::schemas::Pet;

#[endpoint(
    Meta(
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
        Response(Pet, "Operation Successful"),
        Response(400, "Invalid ID supplied"),
        Response(404, "Pet not found"),
        Response(422, "Validation exception"),
        Security(Name("petstore_auth"), Scope("write:pets"), Scope("read:pets"))
    ),
    Method(PUT)
)]
async fn put_pet<'a, 'b>(req: &mut Request<'a, 'b>, res: &mut Response<'a>) -> resty::Result {
    let Json(body): Json<Pet> = req.body().await?;
    res.respond(XML(body)).await?;
    Ok(())
}

#[endpoint(
    Meta(
        Tag("pet"),
        Summary("Add a newper to the store"),
        Request(
            Description("Create a new pet in the store"),
            Schema("application/json", Pet),
            Schema("application/xml", Pet),
            Schema("application/x-www-form-urlencoded", Pet),
            Required
        ),
        // Response(
        //     Code(200),
        //     Description("Successful operation"),
        //     Schema("application/json", Pet),
        //     Schema("application/xml", Pet),
        // ),
        // Response(Code(400), Description("Invalid input")),
        // Response(Code(404), Description("Pet not found")),
        // Response(Code(422), Description("Validation exception")),
        // Response(Default, Description("Unexpected error")),
        Security(Name("petstore_auth"), Scope("write:pets"), Scope("read:pets"))
    ),
    Method(POST)
)]
async fn post_pet<'a, 'b>(req: &mut Request<'a, 'b>, res: &mut Response<'a>) -> resty::Result {
    // res.ok(&"Ok").await?;
    Ok(())
}
