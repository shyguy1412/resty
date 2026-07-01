use resty::{Request, Response, endpoint};

#[endpoint(
    Method(GET),
    Header("Content-Type", "text/html; charset=utf-8"),
    Header("Connection", "keep-alive"),
    Header("Keep-Alive", "timeout=5")
)]
async fn get_user<'a>(req: &mut Request<'a>, res: &mut Response<'a>) -> resty::Result {
    Ok(())
}
