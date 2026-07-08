use std::collections::HashMap;

use crate::{HttpMethod, Request, Response};

/// Type alias for the dyn Trait a Handler function must have
///
/// This is not generally used directly since the `#[endpoint]` macro wraps your
/// async function to comply with this Trait
// type Handler = dyn for<'a> Fn(Request<'a>, Response<'a>) -> crate::EndpointTask<'a> + Sync;
type Handler = dyn for<'a, 'data, 'b> Fn(
        &'b mut Request<'a, 'data>,
        &'b mut Response<'a>,
    ) -> crate::EndpointTask<'b>
    + Sync;

/// A router that routes a path to an endpoint while resolving path parameters
///
/// Any route segment starting with `[` is considered a path parameter
/// The route `%404` is used as fallback if no other route could be found
pub struct Router {
    pub(crate) segments: HashMap<&'static str, Router>,
    pub(crate) endpoints: Vec<(&'static Handler, u16)>,
    pub(crate) middleware: Option<&'static Handler>,
}

impl std::fmt::Debug for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl std::fmt::Display for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_mthod(value: u16) -> Vec<HttpMethod> {
            let mut vec = Vec::with_capacity(16);
            let mut cur = 1u16;

            while cur != 0 && cur <= HttpMethod::TRACE as u16 {
                if cur & value > 0 {
                    vec.push(unsafe { std::mem::transmute(cur) });
                }

                cur = cur << 1;
            }

            vec
        }
        writeln!(f, "Router {{")?;
        for (key, value) in &self.segments {
            let route = format!("{}: {}\n", format!("/{key}").replace("/%", "%"), value);
            let route: String = route.lines().map(|line| format!("  {line}\n")).collect();
            write!(f, "{route}")?;
        }

        for method in fmt_mthod(self.endpoints.iter().fold(0, |a, (.., b)| a | b)) {
            write!(f, "  {method}\n")?;
        }

        write!(
            f,
            "  {}\n",
            self.middleware
                .map(|_| "Middleware: Yes")
                .unwrap_or("Middleware: No")
        )?;

        write!(f, "}}")
    }
}

impl Router {
    pub fn empty() -> Self {
        Self {
            segments: HashMap::new(),
            endpoints: Vec::new(),
            middleware: None,
        }
    }

    pub fn new(route_slices: &[RouteSlice]) -> Self {
        let mut route_table = Router::empty();

        for slice in route_slices {
            route_table.add_route(slice)
        }

        // println!("{route_table}");

        return route_table;
    }

    pub fn add_route(&mut self, (route, handler_or_middleware): &RouteSlice) {
        let mut current_router = self;

        // println!("{route:?}");
        //
        for current_segment in *route {
            let Router { segments, .. } = current_router;

            let key = match current_segment.chars().nth(0).map(|c| c == '[') {
                Some(true) => "%param",
                _ => current_segment,
            };

            current_router = segments.entry(key).or_insert_with(Router::empty);
        }

        match handler_or_middleware {
            HandlerOrMiddleware::Handler(method, handler) => {
                current_router.endpoints.push((*method, *handler))
            }
            HandlerOrMiddleware::Middleware(middleware) => {
                current_router.middleware.replace(*middleware);
            }
        }
    }

    pub fn route<'a>(
        &'a self,
        path: &'a str,
    ) -> Option<(&'a Router, Vec<&'a str>, Vec<&'static Handler>)> {
        let mut path_parameters = vec![];
        let mut middlewares = vec![];
        let mut segments = path
            .strip_prefix("/")
            .unwrap_or(path)
            .split("?")
            .take(1)
            .last()
            .unwrap_or("")
            .split("/");
        let mut route = self;

        route
            .middleware
            .inspect(|middleware| middlewares.push(*middleware));

        while let Some(current_segment) = segments.next() {
            if current_segment == "" {
                continue;
            }

            let dynamic = || {
                route
                    .segments
                    .get("%param")
                    .inspect(|_| path_parameters.push(current_segment))
            };

            let Some(next_route) = route
                .segments
                .get(current_segment)
                .or_else(dynamic)
                .or_else(|| self.segments.get("%404"))
            else {
                return None;
            };

            next_route
                .middleware
                .inspect(|middleware| middlewares.push(*middleware));

            route = next_route;
        }

        Some((route, path_parameters, middlewares))
    }

    pub fn handler<'a>(
        &'a self,
        path: &'a str,
        method: HttpMethod,
    ) -> Option<(Vec<&'static Handler>, Vec<&'a str>)> {
        let (route, params, mut middlewares) = self.route(path)?;
        let handler = route.method(method)?;

        middlewares.push(handler);

        Some((middlewares, params))
    }

    pub fn method(&self, method: HttpMethod) -> Option<&'static Handler> {
        self.endpoints.iter().find_map(|(handler, mask)| {
            match method as u16 & mask > 0 || *mask == HttpMethod::ALL as u16 {
                true => Some(*handler),
                false => None,
            }
        })
    }
}

pub enum HandlerOrMiddleware {
    Handler(&'static Handler, u16),
    Middleware(&'static Handler),
}

/// Type alias for the description of a Route
pub type RouteSlice = (
    &'static [&'static str], // route segments
    HandlerOrMiddleware,
);

// /// Data passed to a handler
// pub struct HandlerData<'a> {
//     pub request: httparse::Request<'a, 'a>,
//     pub path_params: Vec<&'a str>,
//     pub readable: &'a mut Readable,
//     pub writeable: &'a mut Writeable,
// }
