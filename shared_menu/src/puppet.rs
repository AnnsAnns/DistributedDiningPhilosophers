use std::error::Error;

use tokio::net::TcpStream;

use crate::calls::{Calls, Commands, Response};

pub struct Puppet {
    address: String,
}

impl Puppet {
    pub fn new(address: String) -> Self {
        Puppet { address }
    }

    async fn puppet_action(&mut self, command: Commands) -> Response {
        let mut stream = TcpStream::connect(&self.address).await.unwrap();
        println!("Sending command to: {}", self.address);
        let result = self.send_command_to(&mut stream, command).await;
        println!("Received response: {:?}", result);

        match result {
            Ok(r) => r,
            Err(e) => Response::Failure(e.to_string()),
        }
    }
}

impl Calls for Puppet {
    async fn register(&mut self, buf: Vec<u8>) -> Response {
        self.puppet_action(Commands::Register(buf)).await
    }
    
    async fn info(&mut self) -> Response {
        self.puppet_action(Commands::Info).await
    }
}