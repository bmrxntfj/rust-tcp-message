use serde::{Deserialize, Serialize};
use bytes::Bytes;

#[derive(Debug)]
pub struct Message {
    pub id: u16,
    pub body: Bytes,
}
impl Message {
    pub fn new(id: u16, body: Bytes) -> Self {
        Self { id, body }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BusinessRequest {
    pub wait: u8,
    pub order_no: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BusinessResponse {
    pub order_no: String,
    pub status: String,
}
