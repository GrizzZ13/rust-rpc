use std::sync::Arc;

use rpc_core::rpc_macro::service;
use tokio::sync::Mutex;

pub use rpc_core::app::App;

#[derive(Default)]
pub struct Basic {
    pub string: Mutex<String>,
}

#[service]
pub async fn set(state: Arc<Basic>, string: String) -> String {
    let mut guard = state.string.lock().await;
    let old = guard.clone();
    *guard = string;
    old
}

#[service]
pub async fn get(state: Arc<Basic>, _string: String) -> String {
    state.string.lock().await.clone()
}
