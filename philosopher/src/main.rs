use calls::{Calls, Response};
use core::str;
use node::{Node, RegisterType};
use rand::{self, Rng};
use random_names::{random_philosopher_name, random_port};
use restaurant::Restaurant;
use states::States;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use svc::Svc;
use tokio::{net::TcpListener, time::sleep};

use shared_menu::*;

mod svc;
#[derive(Debug, Clone)]
struct Neighbor {
    neighbor: Node,
    request: Option<Node>,
}
#[derive(Debug, Clone)]
struct Philosopher {
    pub public_data: Node,
    #[allow(dead_code)]
    pub right_hand: Option<Node>,
    pub left_hand: Option<Node>,
    pub right_neighbor: Neighbor,
    pub left_neighbor: Neighbor,
    pub id: usize,
    pub restaurant: Restaurant,
    pub waiter: Node,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();
    let ip = std::env::var("IP").expect("IP must be set");
    let port = random_port();
    let username = random_philosopher_name();
    let waiter_ip = std::env::var("WAITER_IP").expect("WAITER_IP must be set");
    let waiter_port = std::env::var("WAITER_PORT").expect("WAITER_PORT must be set");

    let addr = format!("{}:{}", ip, port);

    let listener = TcpListener::bind(addr.clone()).await?;
    println!("Listening on {} as {}", addr, username);

    let data = Philosopher {
        public_data: Node {
            username: username.clone(),
            ip: ip.clone(),
            port,
            of_type: RegisterType::Philosopher,
            state: States::Initializing,
        },
        right_hand: None,
        left_hand: None,
        right_neighbor: Neighbor {
            neighbor: Node {
                username: username.clone(),
                ip: ip.clone(),
                port,
                of_type: RegisterType::Philosopher,
                state: States::Initializing,
            },
            request: None,
        },
        left_neighbor: Neighbor {
            neighbor: Node {
                username: username.clone(),
                ip: ip.clone(),
                port,
                of_type: RegisterType::Philosopher,
                state: States::Initializing,
            },
            request: None,
        },
        id: 0,
        restaurant: Restaurant::default(),
        waiter: Node {
            username: "waiter".to_string(),
            ip: waiter_ip.clone(),
            port: waiter_port.parse().unwrap(),
            of_type: RegisterType::Waiter,
            state: States::WaiterActive,
        },
    };

    let svc = Svc {
        data: Arc::new(Mutex::new(data)),
    };

    // Register with the waiter
    println!("Registering with the waiter");
    let own_data = svc.data.lock().unwrap().public_data.to_bytes();
    let mut waiter = svc.data.lock().unwrap().waiter.clone();

    let response = waiter.register(own_data.clone()).await;
    println!("Response from waiter: {:?}", response);

    // Handle incoming connections
    loop {
        let (stream, _) = listener.accept().await?;
        println!("Accepted connection from: {:?}", stream.peer_addr()?);
        let mut svc_clone2 = svc.clone();
        tokio::task::spawn(async move {
            svc_clone2.connection_handler(stream).await;
        });
    }
}

/// two things that we can do if the mutex won't work like this: tokio mutex that can be held across .await or make cutlery not a container but just a field
/// Philosopher main logic loop
async fn sit_at_table(mut svc: Svc) {
    loop {
        match svc.get_parsed_state().await {
            States::PhilosopherThinking => {
                let rnd_sleep = rand::thread_rng().gen_range(1..=3);
                sleep(Duration::from_secs(rnd_sleep)).await;
                svc.set_state(States::PhilosopherHungry).await;
                println!("state: {:?}", States::PhilosopherHungry);
            }
            States::PhilosopherHungry => {
                let svc_clone = svc.clone();
                request_cutlery(svc_clone).await;
                let mut start_eating = false;
                {
                    let data = svc.data.lock().unwrap();
                    if let Some(_) = data.right_hand {
                        if let Some(_) = data.left_hand {
                            start_eating = true;
                        }
                    }
                }
                // We are forced to it this way to avoid a deadlock,
                // because we can't hold the mutex across .await
                // but we also dont want to manually change the state as
                // other methods rely on the state being set via the svc
                if start_eating {
                    svc.set_state(States::PhilosopherEating).await;
                    println!("state: {:?}", States::PhilosopherEating);
                }
            }
            States::PhilosopherEating => {
                let rnd_sleep = rand::thread_rng().gen_range(1..=3);
                sleep(Duration::from_secs(rnd_sleep)).await;
                //pass cutleries if there are open requests
                let right_cutlery = svc.data.lock().unwrap().right_hand.clone();
                svc.use_cutlery(right_cutlery.clone().unwrap()).await;
                let right_request = svc.data.lock().unwrap().right_neighbor.request.clone();
                match right_request {
                    Some(_) => {
                        svc.data.lock().unwrap().right_hand = None;

                        println!("remembered a request for right cutlery");
                        svc.clean_cutlery(right_cutlery.clone().unwrap()).await;
                        pass_cutlery(svc.clone(), "right".to_string(), right_cutlery.unwrap())
                            .await;
                    }
                    _ => {}
                }
                let left_cutlery = svc.data.lock().unwrap().left_hand.clone();
                svc.use_cutlery(left_cutlery.clone().unwrap()).await;
                let left_request = svc.data.lock().unwrap().left_neighbor.request.clone();
                match left_request {
                    Some(_) => {
                        svc.data.lock().unwrap().left_hand = None;
                        println!("remembered a request for left cutlery");
                        svc.clean_cutlery(left_cutlery.clone().unwrap()).await;
                        pass_cutlery(svc.clone(), "left".to_string(), left_cutlery.unwrap()).await;
                    }
                    _ => {}
                }
                svc.set_state(States::PhilosopherThinking).await;
                println!("state: {:?}", States::PhilosopherThinking);
            }
            _ => (),
        }
        sleep(Duration::from_secs(3)).await;
    }
}

/// Passes left or right cutlery to another philosopher
async fn pass_cutlery(svc: Svc, side: String, mut cutlery: Node) -> Response {
    let response = cutlery.put_down().await;
    if response == Response::Success {
        let neighbors_side;
        let mut neighbor: Node;
        if side == "left" {
            neighbor = svc.data.lock().unwrap().left_neighbor.neighbor.clone();
            neighbors_side = "right".to_string();
        } else {
            neighbor = svc.data.lock().unwrap().right_neighbor.neighbor.clone();
            neighbors_side = "left".to_string();
        }
        println!("passing {} cutlery to {:?}.", side, neighbor);

        let pass_response = neighbor
            .receive_cutlery(cutlery.clone(), neighbors_side)
            .await;
        if pass_response == Response::Success {
            return Response::Success;
        } else {
            if side == "left" {
                svc.data.lock().unwrap().left_hand = Some(cutlery);
            } else {
                svc.data.lock().unwrap().right_hand = Some(cutlery);
            }
        }
    }
    Response::Failure("Couldn't pass the cutlery to the neighbor!".to_string())
}
/// Request missing cutlery from neighboring philosophers
async fn request_cutlery(svc: Svc) -> Response {
    println!("requesting cutlery.");
    // check for needed cutlery
    {
        let test1 = svc.data.lock().unwrap().right_hand.clone();
        let test2 = svc.data.lock().unwrap().left_hand.clone();
        println!("I have right: {:?}, left: {:?}", test1, test2);
    }

    //right
    if svc.data.lock().unwrap().right_hand.is_none() {
        let mut right_neighbor = svc.data.lock().unwrap().right_neighbor.neighbor.clone();
        let own_data = svc.data.lock().unwrap().public_data.clone();
        println!("requesting right from: {:?}", right_neighbor.clone());
        let _right_response = right_neighbor
            .receive_request(own_data, "left".to_string())
            .await;
    }
    //left
    if svc.data.lock().unwrap().left_hand.is_none() {
        let mut left_neighbor: Node = svc.data.lock().unwrap().left_neighbor.neighbor.clone();
        let own_data = svc.data.lock().unwrap().public_data.clone();
        println!("requesting left from: {:?}", left_neighbor.clone());
        let _left_response = left_neighbor
            .receive_request(own_data, "right".to_string())
            .await;
    }
    Response::Success
}
