//! This example is implements a rest api as defined in https://petstore.swagger.io/v2/swagger.json

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    process::ExitCode,
    sync::LazyLock,
};

use resty::{Router, TcpScocket};

mod models;

//IDEA: make router a struct with a router trait and derive macro

#[resty::router(
    FileBased("./routes"),
    Meta(
        Title("Swagger Petstore - OpenAPI 3.0"),
        Description(
            "This is a sample Pet Store Server based on the OpenAPI 3.0 specification. \
            You can find out more about swagger at [https://swagger.io](https://swagger.io). \
            In the third iteration of the pet store, we've switched to the design first approach! \
            You can now help us improve the API whether it's by making changes to the definition itself or to the code. \
            That way, with time, we can improve the API in general, and expose some of the new features in OAS3.\n\n\
            Some useful links:\n\
            - [The Pet Store repository](https://github.com/swagger-api/swagger-petstore)\n\
            - [The source API definition for the Pet Store](https://github.com/swagger-api/swagger-petstore/blob/master/src/main/resources/openapi.yaml)"
        ),
        Version("1.0.27"),
        TermsOfService("http://swagger.io/terms/"),
        Contact(Email("apiteam@swagger.io")),
        License(
            Name("Apache 2.0"),
            Url("http://www.apache.org/licenses/LICENSE-2.0.html")
        ),
        Server(Url("localhost")),
        Tag(
            Name("pet"),
            Description("Everything about your Pets"),
            ExternalDocs(Description("Find out more"), Url("http://swagger.io"))
        ),
        Tag(Name("store"), Description("Access to Petstore orders"),),
        Tag(
            Name("user"),
            Description("Operations about user"),
            ExternalDocs(Description("Find out more about our store"), Url("http://swagger.io"))
        ),
        SecuritySchemes(
            ApiKey(Name("api_key"), In("header")),
            OAuth2(
                Name("petstore_auth"),
                Flows(Implicit(
                    AuthorizationUrl("https://petstore3.swagger.io/oauth/authorize"),
                    Scope("write:pets", "modify pets in your account"),
                    Scope("read:pets", "read your pets")
                ))
            )
        ),
    )
)]
static ROUTER: LazyLock<Router>;

fn main() -> ExitCode {
    println!("{}", *ROUTER);

    const ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3333);
    if let Err(error) = resty::bind::<TcpScocket>(ADDR, &ROUTER) {
        println!("{error:?}");
        return ExitCode::FAILURE;
    }

    println!("Listening on port 3333");

    let _: Vec<_> = std::thread::available_parallelism()
        .ok()
        .map(|n| 0..n.get())
        .unwrap_or(0..1)
        .map(|_| resty::spawn_thread())
        .collect();

    std::thread::park();

    return ExitCode::SUCCESS;
}
