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
        let svc_clone = svc.clone();
        tokio::task::spawn(async move {
            if let Err(err) = handle_request(svc_clone, stream).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(service: Svc, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = vec![0; 1024];
    let n = stream
        .read(&mut buf)
        .await
        .expect("couldn't read from tcp socket");
    register(service.clone(), buf);
    info(service, stream);
    return Ok(());
}

fn register(service: Svc, buf: Vec<u8>) {
    println!("Registering a new node!");

    // Spawn async block to handle the request
    let restaurant = service.restaurant.clone();

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
}

fn info(service: Svc, mut stream: TcpStream) {
    let restaurant = service.restaurant.clone();
    let restaurant = restaurant.lock().expect("closed");
    let restaurant_bytes = restaurant.to_bytes();
    tokio::task::spawn(async move {
        let result = stream.write_all(&restaurant_bytes).await;
        stream.shutdown().await.unwrap();
    });
}

#[derive(Debug, Clone)]
struct Svc {
    restaurant: Arc<Mutex<Restaurant>>,
}
