use crate::models::{Label, Status};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TicketPayload {
    pub title: String,
    pub body: String,
    pub labels: Vec<Label>,
    pub assigned_user: Option<i32>,
    pub status: Status,
}

#[derive(Serialize, Deserialize)]
pub struct FilterPayload {
    pub labels: Option<Vec<Label>>,
    pub assigned_user: Option<i32>,
    pub title: Option<String>,
    pub status: Option<Status>,
}
