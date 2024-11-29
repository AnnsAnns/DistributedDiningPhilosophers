use shared_menu::*;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Clone)]
struct Cutlery {
    pub public_data: Node,
    pub in_use_by: Option<Node>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();
    let ip = std::env::var("IP").expect("IP must be set");
    let port = random_port();
    let username = random_cutlery_name();
    let waiter_ip = std::env::var("WAITER_IP").expect("WAITER_IP must be set");
    let waiter_port = std::env::var("WAITER_PORT").expect("WAITER_PORT must be set");
    let addr = format!("{}:{}", ip, port);

    let listener = TcpListener::bind(addr.clone()).await?;
    println!("Listening on {} as {}", addr, username);

    let data = Cutlery {
        public_data: Node {
            username: username.clone(),
            IP: ip.clone(),
            port: port,
            ofType: RegisterType::Cutlery,
        },
        in_use_by: None,
    };

    // Register with the waiter at the specified IP and port /register
    let waiter_addr = format!("{}:{}", waiter_ip, waiter_port);
    let body = data.public_data.to_bytes();
    let mut stream = TcpStream::connect(&waiter_addr).await?;
    println!("Registering with the waiter at: {}", waiter_addr);
    let result = stream.write_all(&body).await;
    stream.shutdown().await?;
    println!("Registered with the waiter: {:?}", result);

    let svc = Svc {
        data: Arc::new(Mutex::new(data)),
    };

    loop {
        let (stream, _) = listener.accept().await?;
        //let io = TokioIo::new(stream);
        let svc_clone = svc.clone();
        tokio::task::spawn(async move {
            if let Err(err) = handle_request(stream).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = vec![0; 1024];
    let n = stream
        .read(&mut buf)
        .await
        .expect("couldn't read from tcp socket");

    return Ok(());
}

#[derive(Debug, Clone)]
struct Svc {
    data: Arc<Mutex<Cutlery>>,
}

//impl Service<Request<IncomingBody>> for Svc {
//    type Response = Response<Full<Bytes>>;
//    type Error = hyper::Error;
//    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
//
//    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
//        fn mk_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
//            Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
//        }
//        let res = match (req.method(), req.uri().path()) {
//            (&Method::GET, "/") => {
//                let data_copy = self.data.lock().unwrap().public_data.clone();
//                mk_response(format!("{:?}", data_copy))
//            }
//            _ => mk_response("Sorry, I am only a fork :(".into()),
//        };
//
//        Box::pin(async { res })
//    }
//}
