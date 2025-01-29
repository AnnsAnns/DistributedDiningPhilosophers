use tokio::net::TcpStream;

use crate::{
    calls::{Calls, Commands, Response},
    states::States,
};

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
    pub state: States,
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
        // println!("Sending command to: {}", self.get_address());
        let result = self.send_command_to(&mut stream, command).await;
        // println!("Received response: {:?}", result);

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
    async fn initialise(
        &mut self,
        buf: (Node, Node, Option<Node>, Option<Node>),
        id: usize,
    ) -> Response {
        self.puppet_action(Commands::Initialize(buf, id)).await
    }
    async fn clean_cutlery(&mut self, cutlery: Node) -> Response {
        self.puppet_action(Commands::CleanCutlery(cutlery)).await
    }
    async fn use_cutlery(&mut self, cutlery: Node) -> Response {
        self.puppet_action(Commands::UseCutlery(cutlery)).await
    }
    async fn is_dirty(&mut self) -> Response {
        self.puppet_action(Commands::IsDirty).await
    }
    async fn pick_up(&mut self, philosopher: Node) -> Response {
        self.puppet_action(Commands::PickUp(philosopher)).await
    }
    async fn put_down(&mut self) -> Response {
        self.puppet_action(Commands::PutDown).await
    }
    async fn receive_cutlery(&mut self, cutlery: Node, side: String) -> Response {
        self.puppet_action(Commands::ReceiveCutlery(cutlery, side))
            .await
    }
    async fn receive_request(&mut self, philosopher: Node, side: String) -> Response {
        self.puppet_action(Commands::ReceiveRequest(philosopher, side))
            .await
    }
    async fn get_waiter(&self) -> Node {
        panic!("Waiter cannot be called on a node that is not yourself");
    }

    async fn report_state_time(&mut self, _state: States, _time: u64) -> Response {
        self.puppet_action(Commands::ReportStateTime(_state, _time))
            .await
    }

    async fn get_state(&mut self) -> Response {
        self.puppet_action(Commands::GetState).await
    }

    async fn set_state(&mut self, state: States) -> Response {
        self.puppet_action(Commands::SetState(state)).await
    }

    async fn inform_state_update(&mut self, buf: Vec<u8>) -> Response {
        self.puppet_action(Commands::InformStateUpdate(buf)).await
    }
}

#[cfg(test)]
mod tests_node {
    use super::*;
    use crate::COMMAND_LEN;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn test_bytes() {
        let node: Node = Node {
            username: "test".to_string(),
            ip: "127.0.0.2".to_string(),
            port: 6666,
            of_type: RegisterType::Cutlery,
            state: States::Initializing,
        };
        let node_in_bytes = node.to_bytes();
        let node_2 = Node::from_bytes(node_in_bytes);
        assert_eq!(node.ip, node_2.ip);
        assert_eq!(node.port, node_2.port);
        assert_eq!(node.username, node_2.username);
    }
    #[test]
    fn test_address() {
        let node: Node = Node {
            username: "test".to_string(),
            ip: "127.0.0.2".to_string(),
            port: 6666,
            of_type: RegisterType::Cutlery,
            state: States::Initializing,
        };
        let address = node.get_address();
        assert_eq!(address, "127.0.0.2:6666");
    }
    #[tokio::test]
    async fn test_puppet_action() {
        let mut node: Node = Node {
            username: "test".to_string(),
            ip: "127.0.0.2".to_string(),
            port: 6666,
            of_type: RegisterType::Cutlery,
            state: States::Initializing,
        };
        let node_clone = node.clone();
        let listener = TcpListener::bind("127.0.0.2:6666").await.unwrap();
        tokio::task::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut buf = vec![0; COMMAND_LEN];
                let n = stream.read_exact(&mut buf).await.unwrap();
                assert_eq!(n, COMMAND_LEN);
                let response = Response::Return(States::to_bytes(&node_clone.state));
                let mut response_bytes = bincode::serialize(&response).unwrap();
                response_bytes.resize(COMMAND_LEN, 0);

                if let Err(e) = stream.write_all(&response_bytes).await {
                    eprintln!("Failed to write to socket; err = {:?}", e);
                }
            }
        });

        let command = Commands::GetState;
        let response = node.puppet_action(command).await;
        let parsed_resp = match response {
            Response::Return(bytes) => States::from_bytes(bytes),
            _ => States::NotResponding,
        };
        assert_eq!(parsed_resp, States::Initializing);
    }
}
