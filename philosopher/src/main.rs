use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{body, Method};
use hyper::{body::Incoming as IncomingBody, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use std::borrow::Borrow;
use std::future::{Future, IntoFuture};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use shared_menu::*;


#[derive(Debug, Clone)]
struct Philosopher {
    pub public_data: Node,
    pub owned_cutlery: Vec<Node>,
    #[allow(dead_code)] // This is sent to the waiter, but not used in this service
    pub wisdom: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();
    let ip = std::env::var("IP").expect("IP must be set");
    let port = random_port();
    let username = random_philosopher_name();
    let waiter_ip = std::env::var("WAITER_IP").expect("WAITER_IP must be set");
    let waiter_port = std::env::var("WAITER_PORT").expect("WAITER_PORT must be set");
    let wisdom = std::env::var("WISDOM").unwrap_or("The fork is mightier than the spoon.".to_string());

    let addr = format!("{}:{}", ip, port);

    let listener = TcpListener::bind(addr.clone()).await?;
    println!("Listening on http://{} as {}", addr, username);

    let data = Philosopher {
        public_data: Node {
            username: username.clone(),
            IP: ip.clone(),
            port: port,
            ofType: RegisterType::Philosopher,
        },
        owned_cutlery: Vec::new(),
        wisdom,
    };

    // Register with the waiter at the specified IP and port /register
    let waiter_addr = format!("{}:{}", waiter_ip, waiter_port);
    let waiter_addr = format!("http://{}/register", waiter_addr);
    let client = reqwest::Client::new();
    let body = data.public_data.to_bytes();
    println!("Registering with the waiter at: {}", waiter_addr);
    let res = client.post(&waiter_addr).body(body).send().await?;
    println!("Registered with the waiter: {:?}", res);

    let svc = Svc {
        data: Arc::new(Mutex::new(data)),
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
    data: Arc<Mutex<Philosopher>>,
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
            (&Method::GET, "/") => {
                let data_copy = self.data.lock().unwrap().public_data.clone();
                mk_response(format!("{:?}", data_copy))
            }
            _ => mk_response("The path you seek is not found, much like the meaning of life.".into()),
        };

        Box::pin(async { res })
    }
}
