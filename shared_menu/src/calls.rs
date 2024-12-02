

use std::error::Error;

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

use crate::COMMAND_LEN;

/// Response from a call
/// - **Success**: The call was successful
/// - **Failure**: The call failed with the given message
/// - **Return**: The call returned the given data, serialized as a byte vector
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum Response {
    Success,
    Failure(String),
    Return(Vec<u8>),
    NotImpl,
}

/// Commands that can be sent to a node
/// This are treated as calls to the node
/// and are handled by the nodes implementation of the `Calls` trait
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum Commands {
    Register(Vec<u8>),
    Info,
}

/// Trait for a node that can be called
/// This trait is implemented to either be a waiter, cutlery or philosopher
/// which also dictates the implementation of the `Calls` trait
pub trait Calls {
    /// Register a node with the network
    /// The buffer contains the serialized node to be registered
    fn register(&mut self, buf: Vec<u8>) -> Response;

    /// Send info about itself to a node
    fn info(&mut self) -> Response;

    /// Get call from command
    fn get_call(&mut self, command: Commands) -> Response {
        match command {
            Commands::Register(buf) => self.register(buf),
            Commands::Info => self.info(),
            _ => Response::Failure("Unknown command!".to_string()),
        }
    }

    /// Handle a request
    /// The buffer contains the serialized command to be executed
    fn handle_request(&mut self, buf: Vec<u8>) -> Response {
        let command: Commands = bincode::deserialize(&buf).unwrap();
        self.get_call(command)
    }

    async fn receive_bytes(
        &mut self,
        stream: &mut TcpStream,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("Receiving bytes from stream {:?}", stream.peer_addr());

        let mut buf = vec![0; COMMAND_LEN];
        let n = stream.read_exact(&mut buf).await;

        let size = match n {
            Ok(size) => size,
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                return Err(Box::new(e));
            }
        };

        println!("Received {} bytes", size);
        Ok(buf)
    }

    async fn connection_handler(&mut self, mut stream: TcpStream) {
        let buf = match self.receive_bytes(&mut stream).await {
            Ok(buf) => buf,
            Err(e) => {
                eprintln!("Error receiving bytes: {:?}", e);
                return;
            }
        };

        let response = self.handle_request(buf);

        println!("Response: {:?}", response);

        match response {
            Response::Return(bytes) => {
                let _ = stream.write_all(&bytes).await;
            }
            Response::Failure(msg) => {
                eprintln!("Error: {}", msg);
            }
            _ => {}
        }
    }

    async fn send_command_to(
        &mut self,
        stream: &mut TcpStream,
        command: Commands
    ) -> Result<Response, Box<dyn Error>> {
        // Write the command to the stream
        let mut command = command.to_bytes();
        command.resize(COMMAND_LEN, 0);

        if let Err(e) = stream.write_all(&command).await {
            eprintln!("Failed to write to socket; err = {:?}", e);
            return Err(Box::new(e));
        }

        if let Err(e) = stream.flush().await {
            eprintln!("Failed to flush socket; err = {:?}", e);
            return Err(Box::new(e));
        }

        // Read the response from the stream
        let mut buf = vec![0; COMMAND_LEN];
        let n = stream.read_exact(&mut buf).await;
        let size = match n {
            Ok(size) => size,
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                return Err(Box::new(e));
            }
        };

        println!("Received {} bytes", size);
        let response: Response = bincode::deserialize(&buf).unwrap();
        Ok(response)
    }
}

/// Generic implementation of the `Calls` trait for a node
impl Commands {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let command = bincode::deserialize(&bytes).unwrap();
        println!("Received command: {:?}", command);
        command
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}
