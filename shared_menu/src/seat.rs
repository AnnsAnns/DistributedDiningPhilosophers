use crate::node::Node;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Seat {
    own_id: u64,
    philosopher_before: Node,
    philosopher_after: Node,
    cutlery_before: Node,
    cutlery_after: Node,
    total_philosophers: u64,
    total_cutlery: u64,
}

impl Seat {
    pub fn new(
        own_id: u64,
        philosopher_before: Node,
        philosopher_after: Node,
        cutlery_before: Node,
        cutlery_after: Node,
        total_philosophers: u64,
        total_cutlery: u64,
    ) -> Self {
        Self {
            own_id,
            philosopher_before,
            philosopher_after,
            cutlery_before,
            cutlery_after,
            total_philosophers,
            total_cutlery,
        }
    }

    pub fn get_own_id(&self) -> u64 {
        self.own_id
    }

    pub fn get_philosopher_before(&self) -> Node {
        self.philosopher_before.clone()
    }

    pub fn get_philosopher_after(&self) -> Node {
        self.philosopher_after.clone()
    }

    pub fn get_cutlery_before(&self) -> Node {
        self.cutlery_before.clone()
    }

    pub fn get_cutlery_after(&self) -> Node {
        self.cutlery_after.clone()
    }

    pub fn get_total_philosophers(&self) -> u64 {
        self.total_philosophers
    }

    pub fn get_total_cutlery(&self) -> u64 {
        self.total_cutlery
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
