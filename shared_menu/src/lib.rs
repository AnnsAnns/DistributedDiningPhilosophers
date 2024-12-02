use core::num;

use bytes::Bytes;

use puppet::Puppet;

pub mod calls;
pub mod random_names;
pub mod puppet;

pub const COMMAND_LEN: usize = 1024;

///enum containing all waiter functions callable using rpc
pub enum WaiterCalls {
    Register(String, String, u16, RegisterType),
    Info,
    Status,
}
///enum containing all cutlery functions callable using rpc
pub enum CutleryCalls {
    Status,
}
///enum containing all philosoph functions callable using rpc
pub enum PhilosophCalls {
    Status,
}

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
    pub IP: String,
    pub port: u16,
    pub ofType: RegisterType,
}

impl Node {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let node: Node = bincode::deserialize(&bytes).unwrap();
        node
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(&self).unwrap();
        bytes
    }

    pub fn to_puppet(&self) -> Puppet {
        Puppet::new(format!("{}:{}", self.IP, self.port))
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
        let restaurant: Restaurant = bincode::deserialize(&bytes).unwrap();
        restaurant
    }

    pub fn to_bytes(&self) -> Bytes {
        let bytes = bincode::serialize(&self).unwrap();
        Bytes::from(bytes)
    }
}