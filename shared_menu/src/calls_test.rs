use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{node::Node, states::States, COMMAND_LEN};

use crate::calls::{*};

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio::sync::mpsc;
    use tokio::task;

    struct MockNode;

    impl Calls for MockNode {
        async fn get_state(&mut self) -> Response {
            Response::Success
        }

        async fn set_state(&mut self, _state: States) -> Response {
            Response::Success
        }

        async fn get_waiter(&self) -> Node {
            Node::test_new("Waiter")
        }
    }

    #[tokio::test]
    async fn test_get_state() {
        let mut node = MockNode;
        let response = node.get_state().await;
        assert_eq!(response, Response::Success);
    }

    #[tokio::test]
    async fn test_set_state() {
        let mut node = MockNode;
        let response = node.set_state(States::PhilosopherEating).await;
        assert_eq!(response, Response::Success);
    }

    #[tokio::test]
    async fn test_handle_request() {
        let mut node = MockNode;
        let command = Commands::GetState;
        let buf = bincode::serialize(&command).unwrap();
        let response = node.handle_request(buf).await;
        assert_eq!(response, Response::Success);
    }

    #[tokio::test]
    async fn test_connection_handler() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (tx, mut rx) = mpsc::channel(1);

        task::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut node = MockNode;
            node.connection_handler(stream).await;
            tx.send(()).await.unwrap();
        });

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let command = Commands::GetState;
        let mut command_bytes = bincode::serialize(&command).unwrap();
        command_bytes.resize(COMMAND_LEN, 0);
        stream.write_all(&command_bytes).await.unwrap();

        let mut response_bytes = vec![0; COMMAND_LEN];
        stream.read_exact(&mut response_bytes).await.unwrap();
        let response: Response = bincode::deserialize(&response_bytes).unwrap();

        assert_eq!(response, Response::Success);
        rx.recv().await.unwrap();
    }

    #[tokio::test]
    async fn test_send_command_to() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        task::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut node = MockNode;
            node.connection_handler(stream).await;
        });

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut node = MockNode;
        let response = node.send_command_to(&mut stream, Commands::GetState).await.unwrap();

        assert_eq!(response, Response::Success);
    }
}
