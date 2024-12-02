use bytes::Bytes;

use crate::node::Node;

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