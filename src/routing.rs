use std::collections::HashMap;

use crate::parse::{request::Readable, response::Writeable};

/// Type alias for the dyn Trait a Handler function must have
///
/// This is not generally used directly since the `#[endpoint]` macro wraps your
/// async function to comply with this Trait
pub type Handler = dyn for<'a> Fn(&'a mut HandlerData<'a>) -> crate::EndpointTask<'a> + Sync;

/// A router that routes a path to an endpoint while resolving path parameters
///
/// Any route segment starting with `[` is considered a path parameter
/// The route `%404` is used as fallback if no other route could be found
#[derive(Default)]
pub struct Router {
    pub(crate) segments: HashMap<&'static str, Router>,
    pub(crate) endpoints: HashMap<crate::parse::HttpMethod, &'static Handler>,
}

impl std::fmt::Display for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Router {{")?;
        for (key, value) in &self.segments {
            let route = format!("{}: {}\n", format!("/{key}").replace("/%", "%"), value);
            let route: String = route.lines().map(|line| format!("  {line}\n")).collect();
            write!(f, "{route}")?;
        }

        for (method, ..) in &self.endpoints {
            write!(f, "  {method}\n")?;
        }

        write!(f, "}}")
    }
}

impl Router {
    pub fn new(route_slices: &[RouteSlice]) -> Self {
        let mut route_table = Router::default();

        for slice in route_slices {
            route_table.add_route(slice)
        }

        println!("{route_table}");

        return route_table;
    }

    pub fn add_route(&mut self, (route, handler, method): &RouteSlice) {
        let mut current_router = self;

        for current_segment in *route {
            let Router { segments, .. } = current_router;

            let key = match current_segment.chars().nth(0).map(|c| c == '[') {
                Some(true) => "%param",
                _ => current_segment,
            };

            if !segments.contains_key(current_segment) {
                segments.insert(key, Router::default());
            }

            current_router = segments.get_mut(key).unwrap()
        }

        current_router.endpoints.insert(*method, *handler);
    }

    pub fn route<'a>(&'a self, path: &'a str) -> Option<(&'a Router, Vec<&'a str>)> {
        let mut path_parameters = vec![];
        let mut segments = path
            .strip_prefix("/")
            .unwrap_or(path)
            .split("?")
            .take(1)
            .last()
            .unwrap_or("")
            .split("/");
        let mut route = self;

        while let Some(current_segment) = segments.next() {
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

            route = next_route;
        }

        Some((route, path_parameters))
    }
}

/// Type alias for the description of a Route
pub type RouteSlice = (
    &'static [&'static str], // route segments
    &'static Handler,
    crate::parse::HttpMethod,
);

/// Data passed to a handler
pub struct HandlerData<'a> {
    pub request: httparse::Request<'a, 'a>,
    pub path_params: Vec<&'a str>,
    pub readable: &'a mut Readable,
    pub writeable: &'a mut Writeable,
}
