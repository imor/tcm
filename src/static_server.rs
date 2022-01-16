use std::convert::Infallible;
use std::io::Error;
use std::net::SocketAddr;
use std::path::Path;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper_staticfile::Static;
use tokio::sync::oneshot::{Receiver, Sender};

pub(crate) async fn start_server(root_path: &Path, port: u16, server_ready_tx: Sender<()>, shutdown_signal: Receiver<()>) {
    let address = SocketAddr::from(([127, 0, 0, 1], port));

    let make_service = make_service_fn(move |_| {
        let root = Static::new(root_path);
        async { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, root.clone()))) }
    });

    let server = Server::bind(&address).serve(make_service);
    let graceful = server.with_graceful_shutdown(shutdown(shutdown_signal));

    server_ready_tx.send(()).expect("Failed to send server ready signal");
    
    if let Err(e) = graceful.await {
        eprintln!("Failed graceful shutdown. Server error: {}", e);
    }
}

async fn handle_request<B>(req: Request<B>, root: Static) -> Result<Response<Body>, Error> {
    root.serve(req).await
}

async fn shutdown(shutdown_signal: Receiver<()>) {
    shutdown_signal.await.expect("Failed to receive shutdown signal")
}