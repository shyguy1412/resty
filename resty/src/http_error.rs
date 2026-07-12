//! This module auto-generates constants for a bunch of standard HTTP errors

#![cfg(feature = "http_error")]

const NO_HEADERS: &'static [(&'static str, &'static str)] = &[];

macro_rules! http_error {
    ($($ident:ident => $reason:literal),*$(,)?) => ($(
        pub use $ident::ERROR as $ident;

        #[allow(non_snake_case)]
        mod $ident {
            pub const ERROR: crate::NoBody<&$ident> = crate::NoBody(&$ident);
            pub struct $ident;
            impl crate::RestResponse for $ident
            {
                const CODE: u16 = super::code_from_ident(stringify!($ident));
                const REASON: &'static str = $reason;
                const HEADERS: &'static [(&'static str, &'static str)] = super::NO_HEADERS;
            }
        }

    )*)
}

const fn code_from_ident(str: &str) -> u16 {
    let digit_1 = str.as_bytes()[str.len() - 3];
    let digit_2 = str.as_bytes()[str.len() - 2];
    let digit_3 = str.as_bytes()[str.len() - 1];
    let code = &[digit_1, digit_2, digit_3];
    match u16::from_str_radix(unsafe { str::from_utf8_unchecked(code) }, 10) {
        Ok(v) => v,
        Err(_) => 0,
    }
}

http_error! {
    HttpError400 => "Bad Request",
    HttpError401 => "Unauthorized",
    HttpError402 => "Payment Required",
    HttpError403 => "Forbidden",
    HttpError404 => "Not Found",
    HttpError405 => "Method Not Allowed",
    HttpError406 => "Not Acceptable",
    HttpError407 => "Proxy Authentication Required",
    HttpError408 => "Request Timeout",
    HttpError409 => "Conflict",
    HttpError410 => "Gone",
    HttpError411 => "Length Required",
    HttpError412 => "Precondition Failed",
    HttpError413 => "Payload Too Large",
    HttpError414 => "URI Too Long",
    HttpError415 => "Unsupported Media Type",
    HttpError416 => "Range Not Satisfiable",
    HttpError417 => "Expectation Failed",
    HttpError421 => "Misdirected Request",
    HttpError422 => "Unprocessable Entity",
    HttpError423 => "Locked",
    HttpError424 => "Failed Dependency",
    HttpError425 => "Too Early",
    HttpError426 => "Upgrade Required",
    HttpError428 => "Precondition Required",
    HttpError429 => "Too Many Requests",
    HttpError431 => "Request Header Fields Too Large",
    HttpError451 => "Unavailable For Legal Reasons",
    HttpError418 => "I'm a teapot",
    HttpError420 => "Policy Not Fulfilled",
    HttpError444 => "No Response",
    HttpError449 => "The request should be retried after doing the appropriate action",
    HttpError499 => "Client Closed Request",
    HttpError500 => "Internal Server Error",
    HttpError501 => "Not Implemented",
    HttpError502 => "Bad Gateway",
    HttpError503 => "Service Unavailable",
    HttpError504 => "Gateway Timeout",
    HttpError505 => "HTTP Version not supported",
    HttpError506 => "Variant Also Negotiates",
    HttpError507 => "Insufficient Storage",
    HttpError508 => "Loop Detected",
    HttpError509 => "Bandwidth Limit Exceeded",
    HttpError510 => "Not Extended",
    HttpError511 => "Network Authentication Required"
}
