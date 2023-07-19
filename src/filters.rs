use crate::models::{Label, Ticket};

pub fn filter_by_assigned_user(user_id: Option<i32>, ticket: &Ticket) -> bool {
    match user_id {
        Some(user_id) => ticket.assigned_user.unwrap_or(0) == user_id,
        None => true,
    }
}

pub fn filter_by_title(ticket_title: &Option<String>, ticket: &Ticket) -> bool {
    match ticket_title {
        Some(ticket_title) => ticket.title.contains(ticket_title),
        None => true,
    }
}

pub fn filter_by_labels(ticket_labels: &Option<Vec<Label>>, ticket: &Ticket) -> bool {
    match ticket_labels {
        Some(ticket_labels) => ticket_labels.iter().all(|l| ticket.labels.contains(l)),
        None => true,
    }
}
