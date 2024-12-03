use calls::{Calls, Response};
use node::{Node, RegisterType};
use random_names::{random_philosopher_name, random_port};
use restaurant::Restaurant;
use std::{sync::{Arc, Mutex}, time::Duration};
use tokio::net::TcpListener;

use shared_menu::*;

#[derive(Debug, Clone)]
struct Philosopher {
    pub public_data: Node,
    #[allow(dead_code)] 
    pub owned_cutlery: Vec<Node>,
    pub restaurant: Restaurant,
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

    let addr = format!("{}:{}", ip, port);

    let listener = TcpListener::bind(addr.clone()).await?;
    println!("Listening on {} as {}", addr, username);

    let data = Philosopher {
        public_data: Node {
            username: username.clone(),
            ip: ip.clone(),
            port,
            of_type: RegisterType::Philosopher,
        },
        owned_cutlery: Vec::new(),
        restaurant: Restaurant::default(),
        waiter: Node {
            username: "waiter".to_string(),
            ip: waiter_ip.clone(),
            port: waiter_port.parse().unwrap(),
            of_type: RegisterType::Waiter,
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

    // Spawn async info collector
    let svc_copy = svc.clone();
    tokio::spawn(async move {
        let svc = svc_copy;
        loop {
            let mut waiter = svc.data.lock().unwrap().waiter.clone();
            let response = waiter.info().await;

            if let Response::Return(buf) = response {
                let restaurant = Restaurant::from_bytes(buf.into());
                println!("Received restaurant: {:?}", restaurant);
                let mut data = svc.data.lock().unwrap();
                data.restaurant = restaurant;
            } else {
                println!("Waiter returned an error to info request: {:?}", response);
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

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
