
use tokio::net::TcpStream;

use crate::calls::{Calls, Commands, Response};

/// RegisterType is an enum that represents the type of the node that is being registered.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RegisterType {
    Philosopher,
    Cutlery,
    Waiter,
}

/// Node is a struct that represents a node in the network.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub username: String,
    pub ip: String,
    pub port: u16,
    pub of_type: RegisterType,
}

impl Node {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let node: Node = bincode::deserialize(&bytes).unwrap();
        node
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        
        bincode::serialize(&self).unwrap()
    }

    pub fn get_address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    async fn puppet_action(&mut self, command: Commands) -> Response {
        let mut stream = TcpStream::connect(self.get_address()).await.unwrap();
        println!("Sending command to: {}", self.get_address());
        let result = self.send_command_to(&mut stream, command).await;
        println!("Received response: {:?}", result);

        match result {
            Ok(r) => r,
            Err(e) => Response::Failure(e.to_string()),
        }
    }
}

impl Calls for Node {
    async fn register(&mut self, buf: Vec<u8>) -> Response {
        self.puppet_action(Commands::Register(buf)).await
    }
    
    async fn info(&mut self) -> Response {
        self.puppet_action(Commands::Info).await
    }
}