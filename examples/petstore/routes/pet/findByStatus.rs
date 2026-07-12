use std::{sync::LazyLock, thread, time::Duration};

use resty::{Json, Request, Response, endpoint, http_error, *};

use crate::schemas::{self, Pet};

static TEST_SOURCE: LazyLock<EventManager<ServerSentEvent<Json<Pet>>>> =
    LazyLock::new(EventManager::new);

#[endpoint(
    Tag("pet"),
    Summary("Finds Pets by status"),
    Description("Multiple status values can be provided with comma separated strings"),
    Response(200, [Pet]),
    Response(400, "Invalid ID supplied"),
    Response(404, "Pet not found"),
    Response(422, "Validation exception"),
    Security(Name("petstore_auth"), Scope("write:pets"), Scope("read:pets")),
    Method(GET),
    // Router(super::super::ROUTER)
)]
async fn get_by_status<'a, 'b>(req: &mut Request<'a, 'b>, res: &mut Response<'a>) -> resty::Result {
    let sse = TEST_SOURCE.consumer().await;
    let sender = TEST_SOURCE.sender();

    thread::spawn(move || {
        loop {
            let _ = sender.send_blocking(ServerSentEvent {
                event: String::from("pet"),
                body: Json(Pet {
                    id: Some(1),
                    name: String::from("Name"),
                    category: None,
                    photo_urls: None,
                    tags: None,
                    status: Some(schemas::Status::Available),
                }),
            });
            thread::sleep(Duration::from_millis(0));
        }
    });

    res.stream(sse).await.inspect_err(|e| println!("{e}"))?;

    Ok(())
}
