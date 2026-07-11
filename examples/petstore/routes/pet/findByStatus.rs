use resty::{Json, Request, Response, endpoint, http_error};

use crate::schemas::Pet;

#[endpoint(
    Tag("pet"),
    Summary("Finds Pets by status"),
    Description("Multiple status values can be provided with comma separated strings"),
    Response(400, "Invalid ID supplied"),
    Response(404, "Pet not found"),
    Response(422, "Validation exception"),
    Security(Name("petstore_auth"), Scope("write:pets"), Scope("read:pets")),
    Method(GET)
)]
async fn get_by_status<'a, 'b>(req: &mut Request<'a, 'b>, res: &mut Response<'a>) -> resty::Result {
    // let Json(body): Json<Pet> = req.body().await?;

    let vec: Vec<Pet> = Vec::new();
    let resp = Json(vec);
    res.respond(resp).await?;

    Ok(())
}
