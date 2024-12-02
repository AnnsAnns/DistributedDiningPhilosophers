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
    pub waiter: Node,
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
        waiter: Node {
            username: "waiter".to_string(),
            IP: waiter_ip.clone(),
            port: waiter_port.parse().unwrap(),
            ofType: RegisterType::Waiter,
        },
    };

    let mut svc = Svc {
        data: Arc::new(Mutex::new(data)),
    };

    // Register with the waiter
    {
        println!("Registering with the waiter");
        let own_data = svc.data.lock().unwrap().public_data.to_bytes();
        let mut waiter = svc.data.lock().unwrap().waiter.to_puppet();

        let response = waiter.register(own_data.clone()).await;
        println!("Response from waiter: {:?}", response);
    }

    // Handle incoming connections
    loop {
        let (stream, _) = listener.accept().await?;
        println!("Accepted connection from: {:?}", stream.peer_addr()?);
        let mut svc_clone = svc.clone();
        tokio::task::spawn(async move {
            svc_clone.connection_handler(stream).await;
        });
    }
}

impl Calls for Svc {
    async fn register(&mut self, buf: Vec<u8>) -> Response {
        Response::NotImpl
    }

    async fn info(&mut self) -> Response {
        Response::NotImpl
    }
}