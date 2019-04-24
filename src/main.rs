use futures::{Future, Poll};
use tokio::executor::DefaultExecutor;
use tokio::net::tcp::{ConnectFuture, TcpStream};
use tower_grpc::{BoxBody, Request};
use tower_h2::client::{self, Connect, ConnectError, Connection};
use tower_service::Service;
use tower_util::MakeService;
use tower_request_modifier::RequestModifier;

pub mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));
}

use hello_world::HelloRequest;

pub fn main() {
    let _ = ::env_logger::init();


    let client = Client{address: "".into()};
    let say_hello = client.get_service()
        // .and_then(move |conn| {
        //     use hello_world::client::Greeter;

        //     let conn = tower_request_modifier::Builder::new()
        //         .set_origin(uri)
        //         .build(conn)
        //         .unwrap();

        //     Greeter::new(conn)
        // })
        .and_then(|mut client| {

            client
                .say_hello(Request::new(HelloRequest {
                    name: "What is in a name?".to_string(),
                }))
                .map_err(|e| panic!("gRPC request failed; err={:?}", e))
        })
        .and_then(|response| {
            println!("RESPONSE = {:?}", response);
            Ok(())
        })
        .map_err(|e| {
            println!("ERR = {:?}", e);
        });

    tokio::run(say_hello);
}

struct Dst;

impl Service<()> for Dst {
    type Response = TcpStream;
    type Error = ::std::io::Error;
    type Future = ConnectFuture;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(().into())
    }

    fn call(&mut self, _: ()) -> Self::Future {
        let addr = "[::1]:50051".parse().unwrap();
        TcpStream::connect(&addr)
    }
}

#[derive(Clone)]
pub struct Client {
    pub address: String,
}


impl Client {
    pub fn get_service(
        &self,
    ) -> impl Future<
            Item = hello_world::client::Greeter<RequestModifier<Connection<TcpStream, DefaultExecutor, BoxBody>, ()>>,
        Error = ConnectError<std::io::Error>,
        > {
        let uri: http::Uri = format!("http://{}", self.address)
            .parse()
            .expect("could not parse grpc address as uri");
        let h2_settings = Default::default();
        let mut make_client = Connect::new(
            Dst ,
            h2_settings,
            DefaultExecutor::current(),
        );

        make_client.make_service(()).map(|conn| {
            let conn = tower_request_modifier::Builder::new()
                .set_origin(uri)
                .build(conn)
                .unwrap();

            // let hello = hello_world::client::Greeter::new(conn)
            //     .say_hello(Request::new(HelloRequest {
            //         name: "What is in a name?".to_string(),
            //     }));

            hello_world::client::Greeter::new(conn)
        })
    }
}
