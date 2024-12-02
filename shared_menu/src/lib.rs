use bytes::Bytes;

use node::Node;

pub mod calls;
pub mod random_names;
pub mod node;
pub mod restaurant;

pub const COMMAND_LEN: usize = 1024;