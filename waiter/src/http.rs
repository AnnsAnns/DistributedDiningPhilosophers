use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Error, Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use tokio::net::TcpListener;

use crate::Svc;

pub async fn http_server(svc: Svc, addr: SocketAddr) {
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{}", addr);
    loop {
        let (tcp, _) = listener.accept().await.unwrap();
        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(tcp);

        // Spin up a new task in Tokio so we can continue to listen for new TCP connection on the
        // current task without waiting for the processing of the HTTP1 connection we just received
        // to finish
        let task_svc = svc.clone();
        tokio::task::spawn(async move {
            let svc_clone = task_svc.clone();

            let service = service_fn(move |_req| {
                let svc_clone = svc_clone.clone();
                async move {
                    let json = svc_clone.to_json().await;

                    let response = Response::builder()
                        .status(200)
                        .header("Content-Type", "application/json")
                        .body(Full::new(Bytes::from(json)))
                        .unwrap();

                    Ok::<_, Error>(response)
                }
            });

            if let Err(err) = http1::Builder::new()
                .timer(TokioTimer::new())
                .serve_connection(io, service)
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
    }