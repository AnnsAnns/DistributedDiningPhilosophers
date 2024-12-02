use calls::{Calls, Response};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

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
        let mut svc_clone = svc.clone();
        tokio::task::spawn(async move {
            let buf = match svc_clone.receive_bytes(&mut stream).await {
                Ok(buf) => buf,
                Err(e) => {
                    eprintln!("Error receiving bytes: {:?}", e);
                    return;
                }
            };

            let response = svc_clone.handle_request(buf);

            println!("Response: {:?}", response);

            match response {
                Response::Return(bytes) => {
                    let _ = stream.write_all(&bytes).await;
                }
                Response::Failure(msg) => {
                    eprintln!("Error: {}", msg);
                }
                _ => {}
            }
        });
    }
}

#[derive(Debug, Clone)]
struct Svc {
    restaurant: Arc<Mutex<Restaurant>>,
}

impl Calls for Svc {
    fn register(&mut self, buf: Vec<u8>) -> Response {
        // Spawn async block to handle the request
        let restaurant = self.restaurant.clone();

        tokio::task::spawn(async move {
            println!("Handling registration request!");
            let node = Node::from_bytes(buf);
            let mut restaurant = restaurant.lock().unwrap();
            println!("Registering node: {:?}", node);
            match node.ofType {
                RegisterType::Philosopher => restaurant.phillosophers.push(node),
                RegisterType::Cutlery => restaurant.cutlery.push(node),
            }
        });

        Response::Success
    }

    fn info(&mut self) -> Response {
        let restaurant = self.restaurant.clone();
        let restaurant = restaurant.lock().expect("closed");
        let restaurant_bytes = restaurant.to_bytes().to_vec();
        Response::Return(restaurant_bytes)
    }
}
