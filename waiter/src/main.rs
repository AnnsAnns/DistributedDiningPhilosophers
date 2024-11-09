use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{body, Method};
use hyper::{body::Incoming as IncomingBody, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use std::future::{Future, IntoFuture};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use shared_menu::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([172, 18, 0, 2], 3000).into();

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    let svc = Svc {
        restaurant: Arc::new(Mutex::new(Restaurant {
            phillosophers: Vec::new(),
            cutlery: Vec::new(),
        })),
    };

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let svc_clone = svc.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, svc_clone).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

#[derive(Debug, Clone)]
struct Svc {
    restaurant: Arc<Mutex<Restaurant>>,
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        fn mk_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
        }
        let res = match (req.method(), req.uri().path()) {
            (&Method::POST, "/register") => {
                println!("Registering a new node!");

                // Spawn async block to handle the request
                let restaurant = self.restaurant.clone();

                tokio::task::spawn(async move {
                    println!("Handling registration request!");
                    let body = req.collect().await.unwrap().to_bytes();
                    let node = Node::from_bytes(body);
                    let mut restaurant = restaurant.lock().unwrap();
                    println!("Registering node: {:?}", node);
                    match node.ofType {
                        RegisterType::Philosopher => restaurant.phillosophers.push(node),
                        RegisterType::Cutlery => restaurant.cutlery.push(node),
                    }
                });

                mk_response("Registered".into())
            }
            (&Method::GET, "/info") => {
                // Turn restaurant into a byte response
                let restaurant = self.restaurant.lock().unwrap();
                let restaurant_copy = restaurant.clone();
                let restaurant_bytes = restaurant_copy.to_bytes();
                Ok(Response::builder()
                    .body(Full::new(restaurant_bytes))
                    .unwrap())
            }
            (&Method::GET, "/") => {
                let restraurant_copy = self.restaurant.lock().unwrap().clone();
                mk_response(format!("The current restaurant has {} philosophers and {} cutlery!\n\n\nHere is the raw:\n\n{:#?}",
                restraurant_copy.phillosophers.len(),
                restraurant_copy.cutlery.len(),
                restraurant_copy))
            }
            _ => mk_response("Sorry, we don't serve that here!".into()),
        };

        Box::pin(async { res })
    }
}
