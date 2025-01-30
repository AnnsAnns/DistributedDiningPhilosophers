use std::error::Error;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{node::Node, states::States, COMMAND_LEN};

/// Response from a call
/// - **Success**: The call was successful
/// - **Failure**: The call failed with the given message
/// - **Return**: The call returned the given data, serialized as a byte vector
#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub enum Response {
    Success,
    Failure(String),
    Return(Vec<u8>),
    NotFound,
}

/// Commands that can be sent to a node
/// This are treated as calls to the node
/// and are handled by the nodes implementation of the `Calls` trait
/// @TODO: Switch from Vec<u8> to Vec<Node>
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum Commands {
    Register(Vec<u8>), // Register Nodes with the network, e.g. waiter
    Info,              // Request info about a node, e.g. waiter
    Initialize((Node, Node, Option<Node>, Option<Node>), usize),
    CleanCutlery(Node), // Cleans Cutlery
    UseCutlery(Node),   // Makes Cutlery dirty by being used to eat
    IsDirty,
    ReceiveCutlery(Node, String), // A philosoph receivs a piece of cutlery from another philosoph
    ReceiveRequest(Node, String), // A philosopher receives a request for a piece of cutlery they are holding
    PickUp(Node),
    PutDown,
    SetState(States),
    GetState,
    InformStateUpdate(Vec<u8>), // Inform about the state of the node including full node data
    ReportStateTime(States, u64), // Report the time a state was active
}

/// Trait for a node that can be called
/// This trait is implemented to either be a waiter, cutlery or philosopher
/// which also dictates the implementation of the `Calls` trait
#[allow(async_fn_in_trait)]
pub trait Calls {
    /// Get the state of the node
    async fn get_state(&mut self) -> Response;

    /// Informs a node about the state of other nodes
    async fn inform_state_update(&mut self, _buf: Vec<u8>) -> Response {
        Response::NotFound
    }

    /// Returns the state of the node as a `States` enum
    /// If the node is not responding, `States::NotResponding` is returned
    async fn get_parsed_state(&mut self) -> States {
        let response = self.get_state().await;
        match response {
            Response::Return(bytes) => States::from_bytes(bytes),
            _ => States::NotResponding,
        }
    }

    /// Get the waiter node, used to create generic trait methods
    async fn get_waiter(&self) -> Node;

    /// Sets the state of the node
    /// This is used to update the state of the node
    async fn set_state(&mut self, _state: States) -> Response;

    /// Heartbeat to check if the node is still alive
    async fn heartbeat(&self) -> Response {
        Response::Success
    }

    /// Register a node with the network
    /// The buffer contains the serialized node to be registered
    async fn register(&mut self, _buf: Vec<u8>) -> Response {
        Response::NotFound
    }

    /// Report the time a state was active
    /// This is used for statistics
    async fn report_state_time(&mut self, _state: States, _time: u64) -> Response {
        Response::NotFound
    }

    /// Send info about itself to a node
    async fn info(&mut self) -> Response {
        Response::NotFound
    }
    // seats every philosopher
    async fn initialise(
        &mut self,
        _buf: (Node, Node, Option<Node>, Option<Node>),
        _id: usize,
    ) -> Response {
        Response::NotFound
    }
    /// Cleans the cutlery, should be done by philosophers before passing them to someone else
    async fn clean_cutlery(&mut self, _cutlery: Node) -> Response {
        Response::NotFound
    }

    /// Makes the cutlery dirty, should happen when philosophers eat
    async fn use_cutlery(&mut self, _cutlery: Node) -> Response {
        Response::NotFound
    }

    ///checks if the cutlery is dirty
    async fn is_dirty(&mut self) -> Response {
        Response::NotFound
    }

    /// Receive a request for cutlery from neighboring philosophers
    async fn receive_request(&mut self, _philosopher: Node, _side: String) -> Response {
        Response::NotFound
    }

    /// Receives left or right cutlery from another philosopher
    async fn receive_cutlery(&mut self, _cutlery: Node, _side: String) -> Response {
        Response::NotFound
    }
    /// picks up the cutlery, making it owned by the philosopher doing so
    async fn pick_up(&mut self, _philosopher: Node) -> Response {
        Response::NotFound
    }
    /// puts down the cutlery, removing ownership
    async fn put_down(&mut self) -> Response {
        Response::NotFound
    }

    /// Get call from command
    async fn get_call(&mut self, command: Commands) -> Response {
        match command {
            Commands::Register(buf) => self.register(buf).await,
            Commands::Info => self.info().await,
            Commands::Initialize(buf, id) => self.initialise(buf, id).await,
            Commands::CleanCutlery(cutlery) => self.clean_cutlery(cutlery).await,
            Commands::UseCutlery(cutlery) => self.use_cutlery(cutlery).await,
            Commands::IsDirty => self.is_dirty().await,
            Commands::ReceiveRequest(philosopher, side) => {
                self.receive_request(philosopher, side).await
            }
            Commands::ReceiveCutlery(cutlery, side) => self.receive_cutlery(cutlery, side).await,
            Commands::PickUp(philosopher) => self.pick_up(philosopher).await,
            Commands::PutDown => self.put_down().await,
            Commands::SetState(state) => self.set_state(state).await,
            Commands::GetState => self.get_state().await,
            Commands::InformStateUpdate(buf) => self.inform_state_update(buf).await,
            Commands::ReportStateTime(state, time) => self.report_state_time(state, time).await,
        }
    }

    /// Handle a request
    /// The buffer contains the serialized command to be executed
    async fn handle_request(&mut self, buf: Vec<u8>) -> Response {
        let command: Commands = bincode::deserialize(&buf).unwrap();
        println!("Received command: {:?}", command);
        self.get_call(command).await
    }

    /// Receive bytes from a stream
    /// The bytes are read from the stream and returned
    /// as a byte vector
    /// This is used to receive the command from a node
    /// and to receive the response from a node
    /// The buffer is expected to be of length `COMMAND_LEN`
    async fn receive_bytes(
        &mut self,
        stream: &mut TcpStream,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // println!("Receiving bytes from stream {:?}", stream.peer_addr());

        let mut buf = vec![0; COMMAND_LEN];
        let n = stream.read_exact(&mut buf).await;

        let size = match n {
            Ok(size) => size,
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                return Err(Box::new(e));
            }
        };

        //println!("Received {} bytes", size);
        Ok(buf)
    }

    /// Handle a connection
    /// The stream is the connection to the node
    /// The command is read from the stream and handled
    /// The response is then sent back to the node
    async fn connection_handler(&mut self, mut stream: TcpStream) {
        let buf = match self.receive_bytes(&mut stream).await {
            Ok(buf) => buf,
            Err(e) => {
                eprintln!("Error receiving bytes: {:?}", e);
                return;
            }
        };

        let response = self.handle_request(buf).await;

        println!("Returning response: {:?}", response);

        let mut response_bytes = bincode::serialize(&response).unwrap();
        response_bytes.resize(COMMAND_LEN, 0);

        if let Err(e) = stream.write_all(&response_bytes).await {
            eprintln!("Failed to write to socket; err = {:?}", e);
        }
    }

    /// Send a command to a node
    /// The command is serialized and sent to the node
    /// The response is then read from the node and returned
    /// as a `Response` enum
    async fn send_command_to(
        &mut self,
        stream: &mut TcpStream,
        command: Commands,
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
        //println!("Received {} bytes", size);
        let response: Response = bincode::deserialize(&buf).unwrap();
        Ok(response)
    }

    async fn log(&self, message: &str) {
        println!("{}", message);
    }
}

/// Generic implementation of the `Calls` trait for a node
impl Commands {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let command = bincode::deserialize(&bytes).unwrap();
        command
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}