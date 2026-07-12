use resty::{Json, Request, Response, endpoint, http_error::*};

use crate::{
    db::{DB, PetstoreDB},
    schemas::Pet,
};

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

    let db = &mut DB.lock().await;

    let Some(id) = body.id else {
        return res.respond(HttpError400).await;
    };

    let Some(pet_to_update) = db.get_pet_mut(id) else {
        return res.respond(HttpError404).await;
    };

    *pet_to_update = body;

    res.respond(Json(&*pet_to_update)).await?;

    Ok(())
}

#[endpoint(
    Tag("pet"),
    Summary("Add a new pet to the store"),
    Description("Add a new pet to the store"),
    Request(
        Description("Create a new pet in the store"),
        Schema("application/json", Pet),
        Schema("application/xml", Pet),
        Schema("application/x-www-form-urlencoded", Pet),
        Required
    ),
    Response(200, Pet),
    Response(400, "Invalid ID supplied"),
    Response(422, "Validation exception"),
    Security(Name("petstore_auth"), Scope("write:pets"), Scope("read:pets")),
    Method(PUT)
)]
async fn post_pet<'a, 'b>(req: &mut Request<'a, 'b>, res: &mut Response<'a>) -> resty::Result {
    let Json(mut body): Json<Pet> = req.body().await?;

    let db = &mut DB.lock().await;

    let None = body.id else {
        return res.respond(HttpError400).await;
    };

    let id = db.pet_id();

    body.id.replace(id);

    let Some(pet) = db.add_pet(body) else {
        return res.respond(HttpError400).await;
    };

    res.respond(Json(pet)).await?;

    Ok(())
}
