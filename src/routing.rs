use std::{collections::HashMap, sync::LazyLock};

pub type Handler = dyn for<'a> Fn(httparse::Request<'a, 'a>, smol::net::TcpStream) -> crate::EndpointTask<'a>
    + Sync;

#[derive(Default)]
pub struct Route {
    pub(crate) segments: HashMap<&'static str, Route>,
    pub(crate) endpoints: HashMap<crate::parse::HttpMethod, &'static Handler>,
}

impl Route {
    pub fn route<'a>(&'a self, path: &str) -> Option<&'a Route> {
        let mut segments = path.strip_prefix("/").unwrap_or(path).split("/");
        let mut route = self;

        while let Some(current_segment) = segments.next() {
            let Some(next_route) = route.segments.get(current_segment) else {
                return None;
            };
            route = next_route;
        }

        Some(route)
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
pub static ROUTES: [RouteSlice];
pub(crate) static ROUTE_TABLE: LazyLock<Route> = LazyLock::new(build_route_table);

fn build_route_table() -> Route {
    let mut route_table = Route::default();

    for (route, handler, method) in ROUTES {
        let mut current_table = &mut route_table;

        for current_segment in *route {
            let Route { segments, .. } = current_table;

            if !segments.contains_key(current_segment) {
                segments.insert(current_segment, Route::default());
            }

            current_table = segments.get_mut(current_segment).unwrap()
        }

        current_table.endpoints.insert(*method, *handler);
    }

    return route_table;
}
