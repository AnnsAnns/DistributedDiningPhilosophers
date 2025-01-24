use calls::{Calls, Response};
use node::{Node, RegisterType};
use restaurant::Restaurant;
use states::States;
use std::collections::btree_map::Range;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::task;
use tokio::net::TcpListener;

mod http;

use shared_menu::*;

#[derive(Debug, Clone)]
struct Svc {
    restaurant: Arc<Mutex<Restaurant>>,
    visitors: usize,
    fully_booked: Arc<Mutex<bool>>,
    state: States,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get ip and port from env vars
    let ip = std::env::var("WAITER_IP").expect("WAITER_IP env var not set!");
    let port = std::env::var("WAITER_PORT").expect("WAITER_PORT env var not set!");
    let http_port = std::env::var("WAITER_HTTP_PORT").expect("WAITER_HTTP_PORT env var not set!");
    let visitors = std::env::var("VISITORS")
        .expect("VISITORS env var not set!")
        .parse::<usize>()
        .unwrap();

    let addr = format!("{}:{}", ip, port).parse::<SocketAddr>()?;
    let http_addr = format!("{}:{}", ip, http_port).parse::<SocketAddr>()?;

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on tcp://{} for Nodes", addr);

    let svc = Svc {
        restaurant: Arc::new(Mutex::new(Restaurant {
            phillosophers: Vec::new(),
            cutlery: Vec::new(),
            state_stats: HashMap::new(),
            state_times: HashMap::new(),
        })),
        visitors,
        state: States::WaiterActive,
        fully_booked: Arc::new(Mutex::new(false)),
    };

    let server_svc = svc.clone();
    tokio::task::spawn(async move {
        http::http_server(server_svc, http_addr).await;
    });

    loop {
        let (stream, _) = listener.accept().await?;
        println!("Accepted connection from: {:?}", stream.peer_addr()?);
        let mut svc_clone = svc.clone();
        tokio::task::spawn(async move {
            svc_clone.connection_handler(stream).await;
            if !*svc_clone.fully_booked.lock().unwrap()
                && svc_clone.visitors == svc_clone.restaurant.lock().unwrap().phillosophers.len()
                && svc_clone.visitors == svc_clone.restaurant.lock().unwrap().cutlery.len()
            {
                *svc_clone.fully_booked.lock().unwrap() = true;
                let node = Node {
                    username: "dummy".to_string(),
                    ip: "bad".to_string(),
                    port: 0,
                    of_type: RegisterType::Waiter,
                    state: States::Dead,
                };
                svc_clone
                    .initialise((node.clone(), node, None, None), 0)
                    .await;
            };
        });
    }
}

impl Calls for Svc {
    async fn register(&mut self, buf: Vec<u8>) -> Response {
        // Spawn async block to handle the request
        let restaurant = self.restaurant.clone();

        // Spawn async block to handle the registration request
        // We have to use tasks here to properly handle the mutex
        let _ = tokio::task::spawn(async move {
            println!("Handling registration request!");
            let node = Node::from_bytes(buf);
            let mut restaurant = restaurant.lock().unwrap();
            println!("Registering node: {:?}", node);
            match node.of_type {
                RegisterType::Philosopher => restaurant.phillosophers.push(node),
                RegisterType::Cutlery => restaurant.cutlery.push(node),
                _ => println!("Unknown node type!"),
            }
        })
        .await;

        // Spawn async block to inform all nodes of new node
        //let restaurant_copy = self.restaurant.clone();
        //let _ = tokio::task::spawn(async move {
        //    println!("Informing all nodes of new node!");
        //    let restaurant = restaurant_copy.lock().unwrap();
        //    let phillosophers = restaurant.phillosophers.clone();
        //    for node in phillosophers {
        //        let restaurant_bytes = restaurant.clone().to_bytes().to_vec();
        //        tokio::task::spawn(async move {
        //            let mut node = node.clone();
        //            let response = node.register(restaurant_bytes.clone()).await;
        //            println!("Response from node: {:?}", response);
        //        });
        //    }
        //})
        //.await;

        Response::Success
    }

    async fn info(&mut self) -> Response {
        let restaurant = self.restaurant.clone();
        let restaurant = restaurant.lock().expect("closed");
        let restaurant_bytes = restaurant.to_bytes();
        Response::Return(restaurant_bytes)
    }

    async fn initialise(
        &mut self,
        _buf: (Node, Node, Option<Node>, Option<Node>),
        _id: usize,
    ) -> Response {
        println!("START INITIALIZING");
        {
            let last_id = self.visitors - 1;
            let restaurant = self.restaurant.clone();
            let restaurant = restaurant.lock().unwrap().clone();
            let mut first_phil = restaurant.phillosophers[0].clone();

            first_phil
                .initialise(
                    (
                        restaurant.phillosophers[last_id].clone(),
                        restaurant.phillosophers[1].clone(),
                        None,
                        None,
                    ),
                    0,
                )
                .await;
            let phillosophers = restaurant.phillosophers.clone();

            for i in 1..(last_id) {
                let mut phil = phillosophers[i].clone();

                let mut left_cutl = None;
                let mut right_cutl = None;
                if i % 2 != 0 {
                    right_cutl = Some(restaurant.cutlery[i - 1].clone());
                    left_cutl = Some(restaurant.cutlery[i].clone());
                }
                let philos = phillosophers.clone();
                tokio::task::spawn(async move {
                    phil.initialise(
                        (
                            philos[i - 1].clone(),
                            philos[i + 1].clone(),
                            right_cutl,
                            left_cutl,
                        ),
                        i,
                    )
                    .await;
                });
            }
            let phillosophers = restaurant.phillosophers.clone();
            let mut last_phil = phillosophers[last_id].clone();

            let left;
            let right;
            if last_id % 2 == 0 {
                right = None;
                left = Some(restaurant.cutlery[last_id].clone());
            } else {
                right = Some(restaurant.cutlery[last_id - 1].clone());
                left = Some(restaurant.cutlery[last_id].clone());
            }
            last_phil
                .initialise(
                    (
                        phillosophers[last_id - 1].clone(),
                        phillosophers[0].clone(),
                        right,
                        left,
                    ),
                    self.visitors - 1,
                )
                .await;
        }
        println!("DONE INITIALIZING");
        Response::Success
    }

    async fn get_waiter(&self) -> Node {
        panic!("Either the waiter has dysphoria or this should not be called from the waiter 😛");
    }

    async fn get_state(&mut self) -> Response {
        Response::Return(self.state.to_bytes())
    }

    async fn set_state(&mut self, state: states::States) -> Response {
        self.state = state;
        Response::Success
    }

    async fn inform_state_update(&mut self, buf: Vec<u8>) -> Response {
        let node = Node::from_bytes(buf);

        println!("Received state update from: {:?}", node);

        let mut restaurant = self.restaurant.lock().unwrap();

        restaurant.add_state(node.state.clone());

        match node.of_type {
            RegisterType::Philosopher => {
                let mut phillosophers = restaurant.phillosophers.clone();
                for phil in phillosophers.iter_mut() {
                    if phil.username == node.username {
                        *phil = node.clone();
                    }
                }
                restaurant.phillosophers = phillosophers;
            }
            RegisterType::Cutlery => {
                let mut cutlery = restaurant.cutlery.clone();
                for cut in cutlery.iter_mut() {
                    if cut.username == node.username {
                        *cut = node.clone();
                    }
                }
                restaurant.cutlery = cutlery;
            }
            _ => println!("Unknown node type!"),
        }

        Response::Success
    }

    async fn report_state_time(&mut self, state: States, time: u64) -> Response {
        let mut restaurant = self.restaurant.lock().unwrap();
        let count = restaurant.state_times.entry(state).or_insert(0);
        *count += time;
        Response::Success
    }
}

impl Svc {
    /// Transforms the restaurant to a JSON string (Used for frontend)
    async fn to_json(&self) -> String {
        let restaurant = self.restaurant.lock().unwrap().clone();
        serde_json::to_string(&restaurant).unwrap()
    }
}
