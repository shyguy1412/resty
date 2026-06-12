use std::collections::HashMap;

pub type Handler = dyn for<'a> Fn(&'a mut HandlerData<'a>) -> crate::EndpointTask<'a> + Sync;

#[derive(Default)]
pub struct Router {
    pub(crate) segments: HashMap<&'static str, Router>,
    pub(crate) endpoints: HashMap<crate::parse::HttpMethod, &'static Handler>,
}

impl Router {
    pub fn new(route_slices: &[RouteSlice]) -> Self {
        let mut route_table = Router::default();

        for (route, handler, method) in route_slices {
            let mut current_table = &mut route_table;

            for current_segment in *route {
                let Router { segments, .. } = current_table;

                let key = match current_segment.chars().nth(0).map(|c| c == '[') {
                    Some(true) => "@param",
                    _ => current_segment,
                };

                if !segments.contains_key(current_segment) {
                    segments.insert(key, Router::default());
                }

                current_table = segments.get_mut(key).unwrap()
            }

            current_table.endpoints.insert(*method, *handler);
        }

        return route_table;
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
            println!("{current_segment};{:?}", route.segments.keys());
            let next_route = match route.segments.get(current_segment) {
                Some(route) => route,
                None => match route.segments.get("@param") {
                    Some(route) => {
                        path_parameters.push(current_segment);
                        route
                    }
                    None => return None,
                },
            };

            route = next_route;
        }

        Some((route, path_parameters))
    }
}

#[doc(hidden)]
pub type RouteSlice = (
    &'static [&'static str],
    &'static Handler,
    crate::parse::HttpMethod,
);

#[doc(hidden)]
#[linkme::distributed_slice]
pub static FALLBACK: [&'static Handler];

pub struct HandlerData<'a> {
    pub request: httparse::Request<'a, 'a>,
    pub path_params: Vec<&'a str>,
    pub stream: smol::net::TcpStream,
}
