use calls::{Calls, Response};
use states::States;
use svc::Svc;
use core::str;
use node::{Node, RegisterType};
use rand::{self, Rng};
use random_names::{random_philosopher_name, random_port};
use restaurant::Restaurant;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{net::TcpListener, time::sleep};

use shared_menu::*;

mod svc;

#[derive(Debug, Clone)]
struct Philosopher {
    pub public_data: Node,
    #[allow(dead_code)]
    //right 0, left 1
    pub owned_cutlery: Vec<Option<Node>>,
    //right 0, left 1
    pub remembered_requests: Vec<Option<Node>>,
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
        owned_cutlery: vec![None; 2],
        remembered_requests: vec![None; 2],
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
async fn sit_at_table(svc: Svc) {
    loop {
        //thinking
        if matches!(svc.data.lock().unwrap().public_data.state, States::PhilosopherThinking) {
            let rnd_sleep = rand::thread_rng().gen_range(1..=3);
            sleep(Duration::from_secs(rnd_sleep)).await;
            svc.data.lock().unwrap().public_data.state = States::PhilosopherHungry;
            println!("state: {:?}", States::PhilosopherHungry);
        }
        //hungry
        if matches!(svc.data.lock().unwrap().public_data.state, States::PhilosopherHungry) {
            let svc_clone = svc.clone();
            request_cutlery(svc_clone).await;
            {
                let mut data = svc.data.lock().unwrap();
                if let Some(_) = data.owned_cutlery[0] {
                    if let Some(_) = data.owned_cutlery[1] {
                        data.public_data.state = States::PhilosopherEating;
                        println!("state: {:?}", States::PhilosopherEating);
                    }
                }
            }
        }

        //eating
        if matches!(svc.data.lock().unwrap().public_data.state, States::PhilosopherEating) {
            let rnd_sleep = rand::thread_rng().gen_range(1..=3);
            sleep(Duration::from_secs(rnd_sleep)).await;
            //pass cutleries if there are open requests
            let cutlery1 = svc.data.lock().unwrap().owned_cutlery[0].clone();
            let request1 = svc.data.lock().unwrap().remembered_requests[0].clone();
            match request1 {
                Some(_) => {
                    println!("remembered a request for right cutlery");
                    cutlery1
                        .clone()
                        .unwrap()
                        .clean_cutlery(cutlery1.clone().unwrap())
                        .await;
                    pass_cutlery(svc.clone(), "right".to_string()).await;
                }
                None => {
                    cutlery1
                        .clone()
                        .unwrap()
                        .use_cutlery(cutlery1.clone().unwrap())
                        .await;
                }
            }
            let cutlery2 = svc.data.lock().unwrap().owned_cutlery[1].clone();
            let request2 = svc.data.lock().unwrap().remembered_requests[1].clone();
            match request2 {
                Some(_) => {
                    println!("remembered a request for left cutlery");
                    cutlery2
                        .clone()
                        .unwrap()
                        .clean_cutlery(cutlery2.clone().unwrap())
                        .await;
                    pass_cutlery(svc.clone(), "left".to_string()).await;
                }
                None => {
                    cutlery2
                        .clone()
                        .unwrap()
                        .use_cutlery(cutlery2.clone().unwrap())
                        .await;
                }
            }
            svc.data.lock().unwrap().public_data.state = States::PhilosopherThinking;
            println!("state: {:?}", States::PhilosopherThinking);
        }
        sleep(Duration::from_millis(2000)).await;
    }
}

/// Passes left or right cutlery to another philosopher
async fn pass_cutlery(svc: Svc, side: String) -> Response {
    let pos;
    if side == "left".to_owned() {
        pos = 1
    } else if side == "right".to_owned() {
        pos = 0
    } else {
        return Response::NotFound;
    }
    let mut cutlery = svc.data.lock().unwrap().owned_cutlery[pos].clone().unwrap();
    let response = cutlery.put_down().await;
    if response == Response::Success {
        let mut id;
        let last_id;
        {
            let data = svc.data.lock().unwrap();
            id = data.id;
            last_id = data.restaurant.phillosophers.len();
        }
        let neighbors_side;
        if side == "left" {
            id += 1;
            if id == last_id {
                id = 0
            }
            neighbors_side = "right".to_string();
        } else {
            if id == 0 {
                id = last_id - 1
            } else {
                id -= 1;
            }
            neighbors_side = "left".to_string();
        }

        let mut neighbor: Node = svc.data.lock().unwrap().restaurant.phillosophers[id].clone();
        println!("passing cutlery {} to {:?}.", side, neighbor);

        let pass_response = neighbor.receive_cutlery(cutlery, neighbors_side).await;
        if pass_response == Response::Success {
            svc.data.lock().unwrap().owned_cutlery[pos] = None;
            return Response::Success;
        }
    }
    Response::Failure("Couldn't pass the cutlery to the neighbor!".to_string())
}
/// Request missing cutlery from neighboring philosophers
async fn request_cutlery(svc: Svc) -> Response {
    println!("requesting cutlery.");
    // check for needed cutlery
    let owned_cutlery = svc.data.lock().unwrap().owned_cutlery.clone();
    let mut left_id;
    let mut right_id;
    let last_id;
    let id;
    {
        let data = svc.data.lock().unwrap();
        id = data.id;
        left_id = data.id;
        right_id = data.id;
        last_id = data.restaurant.phillosophers.len();
    }

    left_id += 1;
    if left_id == last_id {
        left_id = 0
    }
    if right_id == 0 {
        right_id = last_id - 1
    } else {
        right_id -= 1;
    }
    println!(
        "{} requesting left {} and right {}, I have: {:?}",
        id, left_id, right_id, owned_cutlery
    );

    //right
    match owned_cutlery[0] {
        None => {
            let mut right_neighbor: Node =
                svc.data.lock().unwrap().restaurant.phillosophers[right_id].clone();
            let right_response = right_neighbor
                .receive_request(right_neighbor.clone(), "left".to_string())
                .await;
        }
        Some(_) => (),
    }
    //left
    match owned_cutlery[1] {
        None => {
            let mut left_neighbor: Node =
                svc.data.lock().unwrap().restaurant.phillosophers[left_id].clone();
            let left_response = left_neighbor
                .receive_request(left_neighbor.clone(), "right".to_string())
                .await;
        }
        Some(_) => (),
    }
    Response::Success
}