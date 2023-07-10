use crate::models::Label;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TicketPayload {
    pub title: String,
    pub body: String,
    pub labels: Vec<Label>,
}
