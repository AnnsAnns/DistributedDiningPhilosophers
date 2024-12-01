use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

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
    let wisdom =
        std::env::var("WISDOM").unwrap_or("The fork is mightier than the spoon.".to_string());

    let addr = format!("{}:{}", ip, port);

    let listener = TcpListener::bind(addr.clone()).await?;
    println!("Listening on {} as {}", addr, username);

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
    let body = data.public_data.to_bytes();

    let mut stream = TcpStream::connect(&waiter_addr).await?;
    println!("Registering with the waiter at: {}", waiter_addr);
    let result = stream.write_all(&body).await;
    stream.shutdown().await?;
    println!("Registered with the waiter: {:?}", result);
    // receive info of the restaurant
    let mut buf = vec![0; 1024];
    let n = stream
        .read(&mut buf)
        .await
        .expect("couldn't read from tcp socket");
    let restaurant = Restaurant::from_bytes(buf.into());

    println!(
        "Info erhalten:\nPhilosophen: {:?}\nCutlery: {:?}",
        restaurant.phillosophers, restaurant.cutlery
    );
    let svc = Svc {
        data: Arc::new(Mutex::new(data)),
    };

    loop {
        let (stream, _) = listener.accept().await?;
        //let io = TokioIo::new(stream);
        let svc_clone = svc.clone();
        tokio::task::spawn(async move {
            if let Err(err) = handle_request(stream).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = vec![0; 1024];
    let n = stream
        .read(&mut buf)
        .await
        .expect("couldn't read from tcp socket");

    return Ok(());
}

#[derive(Debug, Clone)]
struct Svc {
    data: Arc<Mutex<Philosopher>>,
}