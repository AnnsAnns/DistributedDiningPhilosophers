use std::collections::HashMap;

use crate::{node::Node, states::States};

/// Restaurant is a struct that represents the restaurant.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq)]
pub struct Restaurant {
    pub phillosophers: Vec<Node>,
    pub cutlery: Vec<Node>,
    pub state_stats: HashMap<States, usize>,
    pub state_times: HashMap<States, u64>,
    pub max_state_time: HashMap<States, u64>,
    pub min_state_time: HashMap<States, u64>,
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
        self.state_times.insert(state.clone(), new_time);

        let max_time = self.max_state_time.entry(state.clone()).or_insert(time);
        if time > *max_time {
            *max_time = time;
        }

        let min_time = self.min_state_time.entry(state.clone()).or_insert(time);
        if time < *min_time {
            *min_time = time;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::states::States;
    use crate::node::Node;

    #[test]
    fn test_from_bytes() {
        let restaurant = Restaurant {
            phillosophers: vec![Node::test_new("Philosopher1")],
            cutlery: vec![Node::test_new("Fork1")],
            state_stats: HashMap::new(),
            state_times: HashMap::new(),
            max_state_time: HashMap::new(),
            min_state_time: HashMap::new(),
        };
        let bytes = restaurant.to_bytes();
        let deserialized_restaurant = Restaurant::from_bytes(bytes);
        assert_eq!(restaurant, deserialized_restaurant);
    }

    #[test]
    fn test_to_bytes() {
        let restaurant = Restaurant {
            phillosophers: vec![Node::test_new("Philosopher1")],
            cutlery: vec![Node::test_new("Fork1")],
            state_stats: HashMap::new(),
            state_times: HashMap::new(),
            max_state_time: HashMap::new(),
            min_state_time: HashMap::new(),
        };
        let bytes = restaurant.to_bytes();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_add_state_time() {
        let mut restaurant = Restaurant::default();
        restaurant.add_state_time(States::PhilosopherEating, 10);
        assert_eq!(*restaurant.state_times.get(&States::PhilosopherEating).unwrap(), 10);
        assert_eq!(*restaurant.max_state_time.get(&States::PhilosopherEating).unwrap(), 10);
        assert_eq!(*restaurant.min_state_time.get(&States::PhilosopherEating).unwrap(), 10);

        restaurant.add_state_time(States::PhilosopherEating, 5);
        assert_eq!(*restaurant.state_times.get(&States::PhilosopherEating).unwrap(), 15);
        assert_eq!(*restaurant.max_state_time.get(&States::PhilosopherEating).unwrap(), 10);
        assert_eq!(*restaurant.min_state_time.get(&States::PhilosopherEating).unwrap(), 5);
    }

    #[test]
    fn test_add_state() {
        let mut restaurant = Restaurant::default();
        restaurant.add_state(States::CutleryClean(true));
        assert_eq!(*restaurant.state_stats.get(&States::CutleryStatsClean).unwrap(), 1);

        restaurant.add_state(States::CutleryDirty(true));
        assert_eq!(*restaurant.state_stats.get(&States::CutleryStatsDirty).unwrap(), 1);

        restaurant.add_state(States::PhilosopherEating);
        assert_eq!(*restaurant.state_stats.get(&States::PhilosopherEating).unwrap(), 1);
    }
}
