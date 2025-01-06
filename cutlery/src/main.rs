use calls::{Calls, Response};
use node::{Node, RegisterType};
use random_names::{random_cutlery_name, random_port};
use shared_menu::*;
use states::States;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[derive(Debug, Clone)]
struct Cutlery {
    pub public_data: Node,
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
            state: States::CutleryClean(false),
        },
        waiter: Node {
            username: "waiter".to_string(),
            ip: waiter_ip.clone(),
            port: waiter_port.parse().unwrap(),
            of_type: RegisterType::Waiter,
            state: States::WaiterActive,
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
    async fn get_waiter(&self) -> Node {
        let data = self.data.lock().unwrap();
        data.waiter.clone()
    }

    ///cleans the cutlery, should be done by philosophers before passing them to someone else
    async fn clean_cutlery(&mut self, _cutlery: Node) -> Response {
        println!("cleaned");

        let is_used = self.data.lock().unwrap().public_data.state.is_used();
        self.set_state(States::CutleryClean(is_used)).await;

        Response::Success
    }
    ///makes the cutlery dirty, should happen when philosophers eat
    async fn use_cutlery(&mut self, _cutlery: Node) -> Response {
        println!("used to eat");
        let is_used = self.data.lock().unwrap().public_data.state.is_used();
        self.set_state(States::CutleryDirty(is_used)).await;

        Response::Success
    }
    
    async fn pick_up(&mut self, philosopher: Node) -> Response {
        println!("picked up by {}", philosopher.username);
        let is_used = self.data.lock().unwrap().public_data.state.is_used();
        if is_used {
            Response::Failure("No nabbing allowed!".to_string())
        } else {
            self.set_state(States::CutleryClean(true)).await;
            Response::Success
        }
    }
    
    async fn put_down(&mut self) -> Response {
        println!("put down.");
        self.set_state(States::CutleryClean(false)).await;
        Response::Success
    }

    async fn is_dirty(&mut self) -> Response {
        println!("checked for dirt.");
        let data = self.data.lock().unwrap();
        Response::Return(vec![data.public_data.state.is_dirty() as u8])
    }
    
    async fn get_state(&mut self) -> Response {
        Response::Return(self.data.lock().unwrap().public_data.state.to_bytes())
    }
    
    async fn set_state(&mut self, state: States) -> Response {
        self.data.lock().unwrap().public_data.state = state;

        // Inform the waiter about the state change
        let own_data = self.data.lock().unwrap().public_data.to_bytes();
        let mut waiter = self.get_waiter().await;
        waiter.inform_state_update(own_data).await
    }

    async fn inform_state_update(&mut self, _buf: Vec<u8>) -> Response {
        Response::Failure("Don't know how to handle this!".to_string())
    }
}
