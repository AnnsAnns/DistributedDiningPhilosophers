use crate::node::Node;

#[derive(Debug, Hash, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum States {
    // General states
    Initializing,
    NotResponding,
    Dead,

    // Philosopher states
    PhilosopherThinking,
    PhilosopherHungry,
    PhilosopherEating,

    // Cutlery states
    CutleryClean(bool),
    CutleryDirty(bool),

    CutleryStatsClean,
    CutleryStatsDirty,

    // Waiter states
    WaiterActive
}

impl States {
    pub fn is_clean(&self) -> bool {
        match self {
            States::CutleryClean(_) => true,
            _ => false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        !self.is_clean() // Trick 17 :P
    }

    /// Returns the node that is used, if any
    /// This is used to check if the cutlery is used by a philosopher
    pub fn is_used(&self) -> bool {
        matches!(self, States::CutleryDirty(true) | States::CutleryClean(true))
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let command = bincode::deserialize(&bytes).unwrap();
        command
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::States;
    use bincode;

    #[test]
    fn test_is_clean() {
        assert!(States::CutleryClean(true).is_clean());
        assert!(States::CutleryClean(false).is_clean());
        assert!(!States::CutleryDirty(true).is_clean());
        assert!(!States::CutleryDirty(false).is_clean());
    }

    #[test]
    fn test_is_dirty() {
        assert!(!States::CutleryClean(true).is_dirty());
        assert!(!States::CutleryClean(false).is_dirty());
        assert!(States::CutleryDirty(true).is_dirty());
        assert!(States::CutleryDirty(false).is_dirty());
    }

    #[test]
    fn test_is_used() {
        assert!(States::CutleryClean(true).is_used());
        assert!(!States::CutleryClean(false).is_used());
        assert!(States::CutleryDirty(true).is_used());
        assert!(!States::CutleryDirty(false).is_used());
    }

    #[test]
    fn test_serialization() {
        let state = States::PhilosopherThinking;
        let bytes = state.to_bytes();
        let deserialized_state = States::from_bytes(bytes);
        assert_eq!(state, deserialized_state);
    }

    #[test]
    fn test_deserialization() {
        let state = States::WaiterActive;
        let bytes = bincode::serialize(&state).unwrap();
        let deserialized_state: States = bincode::deserialize(&bytes).unwrap();
        assert_eq!(state, deserialized_state);
    }
}