use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketPayload {
    pub title: String,
    pub body: String,
}
