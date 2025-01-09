use crate::node::Node;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Seat {
    own_id: u64,
    philosopher_before: Node,
    philosopher_after: Node,
    cutlery_before: Node,
    cutlery_after: Node,
}

impl Seat {
    pub fn new(own_id: u64, philosopher_before: Node, philosopher_after: Node, cutlery_before: Node, cutlery_after: Node) -> Self {
        Self {
            own_id,
            philosopher_before,
            philosopher_after,
            cutlery_before,
            cutlery_after,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let seat: Seat = bincode::deserialize(&bytes).unwrap();
        seat
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(&self).unwrap();
        bytes
    }
}