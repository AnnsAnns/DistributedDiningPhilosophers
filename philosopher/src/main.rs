use std::error::Error;
use std::sync::{Arc, Mutex};
use calls::{Calls, Commands, Response};
use random_names::{random_philosopher_name, random_port};
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

#[derive(Debug, Clone)]
struct Svc {
    data: Arc<Mutex<Philosopher>>,
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

    let mut svc = Svc {
        data: Arc::new(Mutex::new(data)),
    };

    // Register with the waiter at the specified IP and port /register
    let waiter_addr = format!("{}:{}", waiter_ip, waiter_port);

    let mut stream = TcpStream::connect(&waiter_addr).await?;
    println!("Registering with the waiter at: {}", waiter_addr);
    let command = Commands::Register(svc.data.lock().unwrap().public_data.to_bytes());
    let result = svc.send_command_to(&mut stream, command).await;
    println!("Registered with the waiter: {:?}", result);
    stream.shutdown().await?;

    loop {
    }
}

impl Calls for Svc {
    fn register(&mut self, buf: Vec<u8>) -> Response {
        Response::NotImpl
    }

    fn info(&mut self) -> Response {
        Response::NotImpl
    }
}