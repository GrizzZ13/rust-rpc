use std::{future::Future, pin::Pin};

use serde::{de::DeserializeOwned, Serialize};

pub struct Service<State, Req, Res, F>
where
    State: Send + Sync,
    Req: DeserializeOwned + Send,
    Res: Serialize + Send,
    F: Future<Output = Res> + Send,
{
    pub(crate) name: String,
    pub(crate) handle_fn: Pin<Box<fn(State, Req) -> F>>,
}

impl<State, Req, Res, F> Service<State, Req, Res, F>
where
    State: Send + Sync,
    Req: DeserializeOwned + Send,
    Res: Serialize + Send,
    F: Future<Output = Res> + Send,
{
    pub fn new(name: String, handle_fn: fn(State, Req) -> F) -> Self {
        Self {
            name,
            handle_fn: Box::pin(handle_fn),
        }
    }
}
