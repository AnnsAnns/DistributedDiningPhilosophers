use core::num;

use bytes::Bytes;
use rand::Rng;

///enum containing all waiter functions callable using rpc
pub enum WaiterCalls {
    Register(String, String, u16, RegisterType),
    Info,
    Status,
}
///enum containing all cutlery functions callable using rpc
pub enum CutleryCalls {
    Status,
}
///enum containing all philosoph functions callable using rpc
pub enum PhilosophCalls {
    Status,
}

/// RegisterType is an enum that represents the type of the node that is being registered.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RegisterType {
    Philosopher,
    Cutlery,
}

/// Node is a struct that represents a node in the network.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub username: String,
    pub IP: String,
    pub port: u16,
    pub ofType: RegisterType,
}

impl Node {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let node: Node = bincode::deserialize(&bytes).unwrap();
        node
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(&self).unwrap();
        bytes
    }
}

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
        let restaurant: Restaurant = bincode::deserialize(&bytes[..]).unwrap();
        restaurant
    }

    pub fn to_bytes(&self) -> Bytes {
        let bytes = bincode::serialize(&self).unwrap();
        Bytes::from(bytes)
    }
}

// Create a random port number between 3000 and 9000
pub fn random_port() -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen_range(3000..9000)
}

pub fn random_philosopher_name() -> String {
    let names = vec![
        "Aristotle",
        "Plato",
        "Socrates",
        "Kant",
        "Hume",
        "Locke",
        "Descartes",
        "Nietzsche",
        "Wittgenstein",
        "Hegel",
        "Marx",
        "Russell",
        "Heidegger",
        "Kierkegaard",
        "Sartre",
        "Camus",
        "Foucault",
        "Derrida",
        "Deleuze",
        "Zizek",
    ];
    let mut rng = rand::thread_rng();
    let number = rng.gen_range(0..1000);
    let name = format!(
        "{} {} {}",
        names[rng.gen_range(0..names.len())],
        names[rng.gen_range(0..names.len())],
        number
    );
    name.to_string()
}

pub fn random_cutlery_name() -> String {
    let names = vec![
        "Fork",
        "Spoon",
        "Knife",
        "Chopsticks",
        "Spork",
        "Splayd",
        "Trongs",
        "Chork",
        "Knork",
    ];
    let mut rng = rand::thread_rng();
    let name = names[rng.gen_range(0..names.len())];
    let number = rng.gen_range(0..1000);
    format!("{} {}", name, number).to_string()
}
