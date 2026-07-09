use resty::{NoBody, Response};

#[derive(Response)]
struct HttpError400;

impl ::resty::RestResponse<NoBody<HttpError400>> for HttpError400 {
    const CODE: u16 = 400;
    const REASON: &'static str = "Bad Request";
    const HEADERS: &'static [(&'static str, &'static str)] = &[];
}
