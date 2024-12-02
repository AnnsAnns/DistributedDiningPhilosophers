use calls::Calls;
use node::{Node, RegisterType};
use random_names::{random_cutlery_name, random_port};
use shared_menu::*;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Clone)]
struct Cutlery {
    pub public_data: Node,
    pub in_use_by: Option<Node>,
    pub waiter: Node,
}

#[derive(Debug, Clone)]
struct Svc {
    data: Arc<Mutex<Cutlery>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();
    let ip = std::env::var("IP").expect("IP must be set");
    let port = random_port();
    let username = random_cutlery_name();
    let waiter_ip = std::env::var("WAITER_IP").expect("WAITER_IP must be set");
    let waiter_port = std::env::var("WAITER_PORT").expect("WAITER_PORT must be set");
    let addr = format!("{}:{}", ip, port);

    let listener = TcpListener::bind(addr.clone()).await?;
    println!("Listening on {} as {}", addr, username);

    let data = Cutlery {
        public_data: Node {
            username: username.clone(),
            IP: ip.clone(),
            port: port,
            ofType: RegisterType::Cutlery,
        },
        in_use_by: None,
        waiter: Node {
            username: "waiter".to_string(),
            IP: waiter_ip.clone(),
            port: waiter_port.parse().unwrap(),
            ofType: RegisterType::Waiter,
        },
    };

    let svc = Svc {
        data: Arc::new(Mutex::new(data)),
    };

    // Register with the waiter
    println!("Registering with the waiter");
    let own_data = svc.data.lock().unwrap().public_data.to_bytes();
    let mut waiter = svc.data.lock().unwrap().waiter.clone();

    let response = waiter.register(own_data.clone()).await;
    println!("Response from waiter: {:?}", response);

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
    
}
