pub mod calls;
#[cfg(test)]
pub mod calls_test;

pub mod node;
#[cfg(test)]
pub mod node_test;

pub mod random_names;
pub mod restaurant;

#[cfg(test)]
pub mod restaurant_test;

pub mod states;

pub const COMMAND_LEN: usize = 1024;
