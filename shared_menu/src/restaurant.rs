use std::collections::HashMap;

use bytes::Bytes;

use crate::{node::Node, states::States};

/// Restaurant is a struct that represents the restaurant.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub struct Restaurant {
    pub phillosophers: Vec<Node>,
    pub cutlery: Vec<Node>,
    pub state_stats: HashMap<States, usize>,
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

    pub fn add_state(&mut self, state: States) {
        // If the state is CutleryClean or CutleryDirty, we want to add the state with the value true
        // as we only care about the state, not the value of the state
        let state_to_add = {
            match state {
                States::CutleryClean(_) => States::CutleryClean(true),
                States::CutleryDirty(_) => States::CutleryDirty(true),
                s => s,
            }
        };

        println!("Adding state: {:?}", state_to_add);

        let count = self.state_stats.entry(state_to_add).or_insert(0);
        *count += 1;
    }
}