use rpc_lib::*;
use std::net::ToSocketAddrs;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080".to_socket_addrs().unwrap().next().unwrap();
    println!("{}", rpc_call_set(addr, "Hello world".to_string()).await);
    println!("{}", rpc_call_get(addr, "".to_string()).await);
    println!("{}", rpc_call_set(addr, "World Hello".to_string()).await);
    println!("{}", rpc_call_get(addr, "".to_string()).await);
    ()
}
