use tokio::net::TcpStream;

use crate::{
    calls::{Calls, Commands, Response},
    states::States,
};

use crate::node::{*};

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
            port: 5444,
            of_type: RegisterType::Cutlery,
            state: States::Initializing,
        };
        let node_clone = node.clone();
        let listener = TcpListener::bind("127.0.0.2:5444").await.unwrap();
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
