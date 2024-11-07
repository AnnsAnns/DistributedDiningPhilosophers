use bytes::Bytes;

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

impl Node {
    pub fn from_bytes(bytes: Bytes) -> Self {
        let node: Node = bincode::deserialize(&bytes[..]).unwrap();
        node
    }

    pub fn to_bytes(&self) -> Bytes {
        let bytes = bincode::serialize(&self).unwrap();
        Bytes::from(bytes)
    }
}

/// Restaurant is a struct that represents the restaurant.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Restaurant {
    pub phillosophers: Vec<Node>,
    pub cutlery: Vec<Node>,
}

impl Default for Restaurant {
    fn default() -> Self {
        Self {
            phillosophers: Vec::new(),
            cutlery: Vec::new(),
        }
    }
}

impl Restaurant {
    pub fn from_bytes(bytes: Bytes) -> Self {
        let restaurant: Restaurant = bincode::deserialize(&bytes[..]).unwrap();
        restaurant
    }

    pub fn to_bytes(&self) -> Bytes {
        let bytes = bincode::serialize(&self).unwrap();
        Bytes::from(bytes)
    }
}