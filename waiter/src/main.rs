use bytes::{Bytes, BytesMut};
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{body, Method};
use hyper::{body::Incoming as IncomingBody, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use std::error::Error;
use std::future::{Future, IntoFuture};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use shared_menu::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get ip and port from env vars
    let ip = std::env::var("WAITER_IP").expect("WAITER_IP env var not set!");
    let port = std::env::var("WAITER_PORT").expect("WAITER_PORT env var not set!");

    let addr = format!("{}:{}", ip, port).parse::<SocketAddr>()?;

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);

    let svc = Svc {
        restaurant: Arc::new(Mutex::new(Restaurant {
            phillosophers: Vec::new(),
            cutlery: Vec::new(),
        })),
    };

    loop {
        let (mut stream, _) = listener.accept().await?;
        //let io = TokioIo::new(stream);
        let svc_clone = svc.clone();
        tokio::task::spawn(async move {
            if let Err(err) = handle_request(svc_clone, stream).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(service: Svc, mut stream: TcpStream) -> Result<(), Box<dyn Error>>{
    let (reader, writer) = stream.split();
    let mut reader = BufReader::new(reader);
    //let mut writer = BufWriter::new(writer);

    let mut incoming_msg= vec![0;1024];
    stream.readable().await?;
    print!("yay");
    match stream.try_read(&mut incoming_msg) {
        Ok(n) => {
            incoming_msg.truncate(n);
        }
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        }
        Err(e) => {
            return Err(e.into());
        }
    }
    print!("{}",String::from_utf8(incoming_msg)?);
    return Ok(());
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
