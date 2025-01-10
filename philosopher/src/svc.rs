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
        let restaurant = Restaurant::from_bytes(buf);
        println!("Received restaurant update: {:?}", restaurant);
        data.restaurant = restaurant;
        Response::Success
    }

    async fn initialise(
        &mut self,
        buf: (Node, Node, Option<Node>, Option<Node>),
        id: usize,
    ) -> Response {
        self.data.lock().unwrap().id = id;
        let personal_node = self.data.lock().unwrap().public_data.clone();

        // save seat neighbours to variables
        self.data.lock().unwrap().right_neighbor = Neighbor {
            neighbor: buf.0.clone(),
            request: None,
        };
        self.data.lock().unwrap().left_neighbor = Neighbor {
            neighbor: buf.1.clone(),
            request: None,
        };
        println!(
            "neighbours saved as: right: {:?}, left: {:?}",
            buf.0.clone(),
            buf.1.clone()
        );
        // save received cutleries
        match buf.2 {
            Some(mut cutlery) => {
                let response = cutlery.pick_up(personal_node.clone()).await;
                match response {
                    Response::Success => self.data.lock().unwrap().right_hand = Some(cutlery),
                    _ => {
                        return Response::Failure(
                            "Couldn't pick up cutlery during initializing!".to_string(),
                        )
                    }
                }
                println!("grabbed first cutlery...");
            }
            None => {}
        }
        match buf.3 {
            Some(mut cutlery) => {
                let response = cutlery.pick_up(personal_node.clone()).await;
                match response {
                    Response::Success => self.data.lock().unwrap().left_hand = Some(cutlery),
                    _ => {
                        return Response::Failure(
                            "Couldn't pick up cutlery during initializing!".to_string(),
                        )
                    }
                }
                println!("grabbed second cutlery...");
            }
            None => {}
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
        println!("eating...");
        let response = cutlery.use_cutlery(cutlery.clone()).await;
        response
    }

    /// Receives left or right cutlery from another philosopher
    async fn receive_cutlery(&mut self, mut cutlery: Node, side: String) -> Response {
        println!("received cutlery, {}", side);

        let public_data = self.data.lock().unwrap().public_data.clone();
        let response = cutlery.pick_up(public_data).await;
        if response == Response::Success {
            if side == "left" {
                self.data.lock().unwrap().left_hand = Some(cutlery);
            } else if side == "right" {
                self.data.lock().unwrap().right_hand = Some(cutlery);
            } else {
                return Response::NotFound;
            }
            return Response::Success;
        }
        Response::Failure("Couldn't receive cutlery!".to_string())
    }

    /// Receive a request for cutlery from neighboring philosophers
    async fn receive_request(&mut self, philosopher: Node, side: String) -> Response {
        {
            let data = self.data.lock().unwrap();
            let test1 = data.right_hand.clone();
            let test2 = data.left_hand.clone();
            println!("received request, my cutlery: {:?},{:?}", test1, test2);
        }
        let opt_cutlery;
        if side == "left" {
            opt_cutlery = self.data.lock().unwrap().left_hand.clone();
        } else if side == "right" {
            opt_cutlery = self.data.lock().unwrap().right_hand.clone();
        } else {
            return Response::NotFound;
        }
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
                        if matches!(state, States::PhilosopherEating) {
                            if side == "left" {
                                self.data.lock().unwrap().left_neighbor.request = Some(philosopher);
                            } else {
                                self.data.lock().unwrap().right_neighbor.request =
                                    Some(philosopher);
                            }
                        } else {
                            //set hand to None when deciding to pass the cutlery
                            if side == "left" {
                                self.data.lock().unwrap().left_hand = None;
                            } else {
                                self.data.lock().unwrap().right_hand = None;
                            }
                            self.clean_cutlery(cutlery.clone()).await;
                            return pass_cutlery(self.clone(), side, cutlery).await;
                        }
                    }
                    false => {
                        if matches!(state, States::PhilosopherThinking) {
                            //set hand to None when deciding to pass the cutlery
                            if side == "left" {
                                self.data.lock().unwrap().left_hand = None;
                            } else {
                                self.data.lock().unwrap().right_hand = None;
                            }
                            return pass_cutlery(self.clone(), side, cutlery).await;
                        } else {
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
