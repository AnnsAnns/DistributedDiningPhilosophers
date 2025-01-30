use std::collections::HashMap;

use crate::restaurant::{*};

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