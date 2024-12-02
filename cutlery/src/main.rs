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
    };

    loop {
    }
}

#[derive(Debug, Clone)]
struct Svc {
    data: Arc<Mutex<Cutlery>>,
}