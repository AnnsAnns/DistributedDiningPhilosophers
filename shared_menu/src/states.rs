use crate::node::Node;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
}