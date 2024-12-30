use calls::{Calls, Response};
use node::Node;
use states::States;
use restaurant::Restaurant;
use std::{
    sync::{Arc, Mutex},
};

use shared_menu::*;

use crate::{pass_cutlery, sit_at_table, Philosopher};

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
        //get info before initializing!
        {
            let mut data = self.data.lock().unwrap();
            let restaurant = Restaurant::from_bytes(buf.into());
            data.restaurant = restaurant;
        }

        self.data.lock().unwrap().id = id;
        let personal_node = self.data.lock().unwrap().public_data.clone();

        if id < (self.data.lock().unwrap().restaurant.phillosophers.len() - 1) {
            let mut cutlery1 = self.data.lock().unwrap().restaurant.cutlery[id].clone();
            let response1 = cutlery1.pick_up(personal_node.clone()).await;
            match response1 {
                Response::Success => self.data.lock().unwrap().owned_cutlery[1] = Some(cutlery1),
                _ => {
                    return Response::Failure(
                        "Couldn't pick up cutlery during initializing!".to_string(),
                    )
                }
            }
        }
        println!(
            "grabbed one...next: {}",
            self.data.lock().unwrap().restaurant.cutlery.len() - 1
        );

        if id == 0 {
            let last_id = self.data.lock().unwrap().restaurant.cutlery.len() - 1;
            let mut cutlery2 = self.data.lock().unwrap().restaurant.cutlery[last_id].clone();
            let response2 = cutlery2.pick_up(personal_node).await;
            match response2 {
                Response::Success => self.data.lock().unwrap().owned_cutlery[0] = Some(cutlery2),
                _ => {
                    return Response::Failure(
                        "Couldn't pick up cutlery during initializing!".to_string(),
                    )
                }
            }
        }
        println!("done grabbing cutlery!");
        //start Philosopher main logic loop
        self.data.lock().unwrap().public_data.state = States::PhilosopherThinking;
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
        let pos1;
        let pos2;
        if side == "left".to_owned() {
            pos1 = 1;
            pos2 = 0;
        } else if side == "right".to_owned() {
            pos1 = 0;
            pos2 = 1;
        } else {
            return Response::NotFound;
        }
        let public_data = self.data.lock().unwrap().public_data.clone();
        let response = cutlery.pick_up(public_data).await;
        if response == Response::Success {
            let mut data = self.data.lock().unwrap();
            data.owned_cutlery[pos1] = Some(cutlery);
            if let Some(_) = data.owned_cutlery[pos2] {
                data.public_data.state = States::PhilosopherEating;
                println!("state: {:?}", States::PhilosopherEating);
            }
            return Response::Success;
        }
        Response::Failure("Couldn't receive cutlery!".to_string())
    }

    /// Receive a request for cutlery from neighboring philosophers
    async fn receive_request(&mut self, philosopher: Node, side: String) -> Response {
        let pos;
        if side == "left".to_owned() {
            pos = 1
        } else if side == "right".to_owned() {
            pos = 0
        } else {
            return Response::NotFound;
        }
        let opt_cutlery = self.data.lock().unwrap().owned_cutlery[pos].clone(); //panicked while unwrapping this multiple times
        println!(
            "received request, my cutlery: {:?}",
            self.data.lock().unwrap().owned_cutlery
        );
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
                if is_dirty {
                    if !matches!(self.data.lock().unwrap().public_data.state, States::PhilosopherEating) {
                        cutlery.clean_cutlery(cutlery.clone()).await;
                        return pass_cutlery(self.clone(), side).await;
                    }
                } else {
                    if !matches!(self.data.lock().unwrap().public_data.state, States::PhilosopherThinking) {
                        self.data.lock().unwrap().remembered_requests[pos] = Some(philosopher);
                    }
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
        Response::Success
    }
}