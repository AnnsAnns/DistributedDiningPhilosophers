
/// Response from a call
/// - **Success**: The call was successful
/// - **Failure**: The call failed with the given message
/// - **Return**: The call returned the given data, serialized as a byte vector
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum Response {
    Success,
    Failure(String),
    Return(Vec<u8>),
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
            _ => {
                Response::Failure("Unknown command!".to_string())
            }
        }
    }
}

/// Generic implementation of the `Calls` trait for a node
impl Commands {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let command: Commands = bincode::deserialize(&bytes).unwrap();
        command
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(&self).unwrap();
        bytes
    }
}