/// RegisterType is an enum that represents the type of the node that is being registered.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RegisterType {
    Philosopher,
    Cutlery,
}

/// Node is a struct that represents a node in the network.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub username: String,
    pub IP: String,
    pub port: u16,
    pub ofType: RegisterType,
}

/// Restaurant is a struct that represents the restaurant.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Restaurant {
    pub phillosophers: Vec<Node>,
    pub cutlery: Vec<Node>,
}