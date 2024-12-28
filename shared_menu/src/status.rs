use crate::node::Node;

#[derive(Debug, Clone)]
pub enum PhilosopherStates {
    Initializing,
    Thinking,
    Hungry,
    Eating,
}

#[derive(Debug, Clone)]
pub enum CutleryStatus {
    Clean(Option<Node>),
    Dirty(Option<Node>),
}

impl CutleryStatus {
    pub fn is_clean(&self) -> bool {
        match self {
            CutleryStatus::Clean(_) => true,
            _ => false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        !self.is_clean() // Trick 17 :P
    }

    /// Returns the node that is used, if any
    /// This is used to check if the cutlery is used by a philosopher
    pub fn is_used(&self) -> Option<Node> {
        match self {
            CutleryStatus::Clean(Some(node)) => Some(node.clone()),
            CutleryStatus::Dirty(Some(node)) => Some(node.clone()),
            _ => None,
        }
    }
}