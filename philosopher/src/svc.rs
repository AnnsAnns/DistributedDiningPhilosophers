use calls::{Calls, Response};
use node::Node;
use restaurant::Restaurant;
use states::States;
use std::sync::{Arc, Mutex};

use shared_menu::*;

use crate::{pass_cutlery, sit_at_table, Neighbor, Philosopher};

#[derive(Debug, Clone)]
pub struct Svc {
    pub data: Arc<Mutex<Philosopher>>,
}

impl Calls for Svc {
    async fn get_waiter(&self) -> Node {
        let data = self.data.lock().unwrap();
        data.waiter.clone()
    }

    async fn register(&mut self, buf: Vec<u8>) -> Response {
        let mut data = self.data.lock().unwrap();
        let restaurant = Restaurant::from_bytes(buf.into());
        println!("Received restaurant update: {:?}", restaurant);
        data.restaurant = restaurant;
        Response::Success
    }

    async fn initialise(&mut self, buf: Vec<u8>, id: usize) -> Response {
        let last_id;
        {
            let mut data = self.data.lock().unwrap();
            let restaurant = Restaurant::from_bytes(buf.into());
            data.restaurant = restaurant;
            last_id = data.restaurant.phillosophers.len() - 1;
        }

        self.data.lock().unwrap().id = id;
        let personal_node = self.data.lock().unwrap().public_data.clone();

        {
            let philosophers = self.data.lock().unwrap().restaurant.phillosophers.clone();
            let mut left_id = id + 1;
            let mut right_id = id;
            if left_id > last_id {
                left_id = 0
            }
            if right_id == 0 {
                right_id = last_id
            } else {
                right_id -= 1;
            }
            self.data.lock().unwrap().right_neighbor = Neighbor {
                neighbor: philosophers[right_id].clone(),
                request: None,
            };
            self.data.lock().unwrap().left_neighbor = Neighbor {
                neighbor: philosophers[left_id].clone(),
                request: None,
            };
            println!(
                "neighbours saved as: left: {:?},right: {:?}",
                philosophers[left_id].clone(),
                philosophers[right_id].clone()
            );
        }

        if id < (self.data.lock().unwrap().restaurant.phillosophers.len() - 1) {
            let mut cutlery1 = self.data.lock().unwrap().restaurant.cutlery[id].clone();
            let response1 = cutlery1.pick_up(personal_node.clone()).await;
            match response1 {
                Response::Success => self.data.lock().unwrap().left_hand = Some(cutlery1),
                _ => {
                    return Response::Failure(
                        "Couldn't pick up cutlery during initializing!".to_string(),
                    )
                }
            }
            println!("grabbed first cutlery...");
        }

        if id == 0 {
            let last_id = self.data.lock().unwrap().restaurant.cutlery.len() - 1;
            let mut cutlery2 = self.data.lock().unwrap().restaurant.cutlery[last_id].clone();
            let response2 = cutlery2.pick_up(personal_node).await;
            match response2 {
                Response::Success => self.data.lock().unwrap().right_hand = Some(cutlery2),
                _ => {
                    return Response::Failure(
                        "Couldn't pick up cutlery during initializing!".to_string(),
                    )
                }
            }
            println!("grabbed second cutlery...",);
        }
        println!("done grabbing cutlery!");
        //start Philosopher main logic loop
        self.set_state(States::PhilosopherThinking).await;
        println!("state: {:?}", States::PhilosopherThinking);
        let svc_clone = self.clone();
        tokio::task::spawn(async move {
            println!("seated.");
            sit_at_table(svc_clone).await;
        });
        Response::Success
    }

    ///cleans the cutlery, should be done by philosophers before passing them to someone else
    async fn clean_cutlery(&mut self, mut cutlery: Node) -> Response {
        println!("cleaning...");
        let response = cutlery.clean_cutlery(cutlery.clone()).await;
        response
    }

    ///makes the cutlery dirty, should happen when philosophers eat
    async fn use_cutlery(&mut self, mut cutlery: Node) -> Response {
        println!("om nom nom...");
        let response = cutlery.use_cutlery(cutlery.clone()).await;
        response
    }

    /// Receives left or right cutlery from another philosopher
    async fn receive_cutlery(&mut self, mut cutlery: Node, side: String) -> Response {
        println!("received cutlery, {}", side);

        let public_data = self.data.lock().unwrap().public_data.clone();
        let response = cutlery.pick_up(public_data).await;
        if response == Response::Success {
            let other_hand;
            if side == "left" {
                self.data.lock().unwrap().left_hand = Some(cutlery);
                other_hand = self.data.lock().unwrap().right_hand.clone();
            } else if side == "right" {
                self.data.lock().unwrap().right_hand = Some(cutlery);
                other_hand = self.data.lock().unwrap().left_hand.clone();
            } else {
                return Response::NotFound;
            }
            // Avoid deadlock by getting the cutlery before setting the state
            if let Some(_) = other_hand {
                self.set_state(States::PhilosopherEating).await;
                println!("state: {:?}", States::PhilosopherEating);
            }
            return Response::Success;
        }
        Response::Failure("Couldn't receive cutlery!".to_string())
    }

    /// Receive a request for cutlery from neighboring philosophers
    async fn receive_request(&mut self, philosopher: Node, side: String) -> Response {
        let opt_cutlery;
        if side == "left" {
            opt_cutlery = self.data.lock().unwrap().left_hand.clone();
        } else if side == "right" {
            opt_cutlery = self.data.lock().unwrap().right_hand.clone();
        } else {
            return Response::NotFound;
        }
        let test1 = self.data.lock().unwrap().right_hand.clone();
        let test2 = self.data.lock().unwrap().left_hand.clone();
        println!("received request, my cutlery: {:?},{:?}", test1, test2);
        let mut cutlery = match opt_cutlery {
            None => {
                println! {"Couldn't find cutlery I was supposed to have!"}
                return Response::Failure("verloren!".to_string());
            }
            Some(cutl) => cutl,
        };
        match cutlery.is_dirty().await {
            Response::Return(result) => {
                let is_dirty: bool = result[0] == 1;
                let state = self.get_parsed_state().await;

                match is_dirty {
                    true => {
                        if !matches!(state, States::PhilosopherEating) {
                            cutlery.clean_cutlery(cutlery.clone()).await;
                            return pass_cutlery(self.clone(), side).await;
                        } else {
                            if side == "left" {
                                self.data.lock().unwrap().left_neighbor.request = Some(philosopher);
                            } else {
                                self.data.lock().unwrap().right_neighbor.request =
                                    Some(philosopher);
                            }
                        }
                    }
                    false => {
                        if !matches!(state, States::PhilosopherThinking) {
                            if side == "left" {
                                self.data.lock().unwrap().left_neighbor.request = Some(philosopher);
                            } else {
                                self.data.lock().unwrap().right_neighbor.request =
                                    Some(philosopher);
                            }
                        }
                    }
                    _ => (),
                }
            }
            _ => {
                return Response::Failure("Didn't receive valid resonse from is_dirty!".to_string())
            }
        }
        Response::Success
    }

    async fn get_state(&mut self) -> Response {
        Response::Return(self.data.lock().unwrap().public_data.state.to_bytes())
    }

    async fn set_state(&mut self, _state: States) -> Response {
        self.data.lock().unwrap().public_data.state = _state;

        // Inform the waiter about the state change
        let own_data = self.data.lock().unwrap().public_data.to_bytes();
        let mut waiter = self.get_waiter().await;
        waiter.inform_state_update(own_data).await
    }

    async fn inform_state_update(&mut self, _buf: Vec<u8>) -> Response {
        Response::Failure("Don't know how to handle this!".to_string())
    }
}
