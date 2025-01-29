use std::collections::HashMap;

use crate::{node::Node, states::States};

/// Restaurant is a struct that represents the restaurant.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct Restaurant {
    pub phillosophers: Vec<Node>,
    pub cutlery: Vec<Node>,
    pub state_stats: HashMap<States, usize>,
    pub state_times: HashMap<States, u64>,
}

impl Restaurant {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let restaurant: Restaurant = bincode::deserialize(&bytes).unwrap();
        restaurant
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(&self).unwrap();
        bytes
    }

    pub fn add_state_time(&mut self, state: States, time: u64) {
        println!("Adding time to state: {:?}, time: {}", state, time);
        let previous_time = self.state_times.get(&state).unwrap_or(&0);
        let new_time = previous_time + time;
        self.state_times.insert(state, new_time);
    }

    pub fn add_state(&mut self, state: States) {
        // Serialization is not working properly when the enum value contains its own value
        let state_to_add = {
            match state {
                States::CutleryClean(_) => States::CutleryStatsClean,
                States::CutleryDirty(_) => States::CutleryStatsDirty,
                s => s,
            }
        };

        println!("Adding state: {:?}", state_to_add);

        let count = self.state_stats.entry(state_to_add).or_insert(0);
        *count += 1;
    }
}
