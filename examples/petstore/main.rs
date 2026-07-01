//! This example is implements a rest api as defined in https://petstore.swagger.io/v2/swagger.json

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    process::ExitCode,
    sync::LazyLock,
};

use resty::{Router, TcpScocket};

mod models;

#[resty::router(
    FileBased("./routes"),
    Meta(
        Description(
            "This is a sample server Petstore server.\n\
            You can find out more about Swagger at [http://swagger.io](http://swagger.io)\
            or on [irc.freenode.net, #swagger](http://swagger.io/irc/).\n\
            For this sample, you can use the api key `special-key` to test the authorization filters."
        ),
        Version("1.0.7"),
        Title("Swagger Petstore"),
        TermsOfService("http://swagger.io/terms/"),
        Contact(Email("apiteam@swagger.io")),
        License(
            Name("Apache 2.0"),
            Url("http://www.apache.org/licenses/LICENSE-2.0.html")
        ),
        Host("localhost"),
        BasePath("/v2"),
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
        Scheme("http"),
        Scheme("https")
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
