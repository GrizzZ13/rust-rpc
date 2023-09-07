use std::{net::ToSocketAddrs, sync::Arc};

use rpc_lib::*;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080".to_socket_addrs().unwrap().next().unwrap();
    let mut app = App::new(Default::default());
    app.service(get::service()).service(set::service());
    for name in app.services() {
        println!("{}", name);
    }
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let app = Arc::new(app);
    let handle = tokio::spawn(app.serve(addr, receiver));
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    sender.send(()).unwrap();
    let _a = handle.await;
}
