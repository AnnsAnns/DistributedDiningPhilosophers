use calls::{Calls, Response};
use node::{Node, RegisterType};
use random_names::{random_cutlery_name, random_port};
use shared_menu::*;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[derive(Debug, Clone)]
struct Cutlery {
    pub public_data: Node,
    #[allow(dead_code)]
    pub in_use_by: Option<Node>,
    pub dirty: bool,
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
            ip: ip.clone(),
            port,
            of_type: RegisterType::Cutlery,
        },
        in_use_by: None,
        dirty: true,
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
    ///cleans the cutlery, should be done by philosophers before passing them to someone else
    async fn clean_cutlery(&mut self, _cutlery: Node) -> Response {
        println!("cleaned");

        let mut data = self.data.lock().unwrap();
        data.dirty = false;

        Response::Success
    }
    ///makes the cutlery dirty, should happen when philosophers eat
    async fn use_cutlery(&mut self, _cutlery: Node) -> Response {
        println!("used to eat");
        let mut data = self.data.lock().unwrap();
        data.dirty = true;

        Response::Success
    }
    async fn pick_up(&mut self, philosopher: Node) -> Response {
        println!("picked up by {}", philosopher.username);
        let mut data = self.data.lock().unwrap();
        match data.in_use_by {
            Some(_) => Response::Failure("No nabbing allowed!".to_string()),
            None => {
                data.in_use_by = Some(philosopher);
                Response::Success
            }
        }
    }
    async fn put_down(&mut self) -> Response {
        println!("put down.");
        let mut data = self.data.lock().unwrap();
        data.in_use_by = None;
        Response::Success
    }
    async fn is_dirty(&mut self) -> Response {
        println!("checked for dirt.");
        let data = self.data.lock().unwrap();
        match data.dirty {
            true => Response::Return("true".as_bytes().to_vec()),
            false => Response::Return("false".as_bytes().to_vec()),
        }
    }
}
