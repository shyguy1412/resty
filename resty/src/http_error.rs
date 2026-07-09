#![cfg(feature = "http_error")]

use resty_macros::{error_code_to_ident, error_code_to_struct};

const NO_HEADERS: &'static [(&'static str, &'static str)] = &[];

macro_rules! http_error {
    ($($code:literal => $reason:literal),*$(,)?) => ($(
        error_code_to_struct!($code);

        impl crate::RestResponse<crate::NoBody<error_code_to_ident!($code)>>
            for error_code_to_ident!($code)
        {
            const CODE: u16 = $code;
            const REASON: &'static str = $reason;
            const HEADERS: &'static [(&'static str, &'static str)] = NO_HEADERS;
        }
    )*)
}

http_error! {
    400 => "Bad Request",
    401 => "Unauthorized",
    402 => "Payment Required",
    403 => "Forbidden",
    404 => "Not Found",
    405 => "Method Not Allowed",
    406 => "Not Acceptable",
    407 => "Proxy Authentication Required",
    408 => "Request Timeout",
    409 => "Conflict",
    410 => "Gone",
    411 => "Length Required",
    412 => "Precondition Failed",
    413 => "Payload Too Large",
    414 => "URI Too Long",
    415 => "Unsupported Media Type",
    416 => "Range Not Satisfiable",
    417 => "Expectation Failed",
    421 => "Misdirected Request",
    422 => "Unprocessable Entity",
    423 => "Locked",
    424 => "Failed Dependency",
    425 => "Too Early",
    426 => "Upgrade Required",
    428 => "Precondition Required",
    429 => "Too Many Requests",
    431 => "Request Header Fields Too Large",
    451 => "Unavailable For Legal Reasons",
    418 => "I'm a teapot",
    420 => "Policy Not Fulfilled",
    444 => "No Response",
    449 => "The request should be retried after doing the appropriate action",
    499 => "Client Closed Request",
    500 => "Internal Server Error",
    501 => "Not Implemented",
    502 => "Bad Gateway",
    503 => "Service Unavailable",
    504 => "Gateway Timeout",
    505 => "HTTP Version not supported",
    506 => "Variant Also Negotiates",
    507 => "Insufficient Storage",
    508 => "Loop Detected",
    509 => "Bandwidth Limit Exceeded",
    510 => "Not Extended",
    511 => "Network Authentication Required"
}
