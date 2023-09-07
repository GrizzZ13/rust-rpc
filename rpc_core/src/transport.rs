use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub name: String,
    pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub payload: Vec<u8>,
}
