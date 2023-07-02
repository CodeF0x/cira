mod database;
mod models;
mod payloads;
mod schema;

use crate::database::{create_ticket, establish_connection};
use crate::payloads::TicketPayload;
use actix_web::http::StatusCode;
use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use serde_json;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    HttpServer::new(|| App::new().service(create))
        .bind(("localhost", 8080))?
        .run()
        .await
}

#[post("/tickets")]
async fn create(req_body: String) -> impl Responder {
    let mut connection = establish_connection();

    let payload: TicketPayload = serde_json::from_str(&req_body).unwrap();

    match create_ticket(&mut connection, payload.title, payload.body) {
        Ok(_) => HttpResponse::new(StatusCode::CREATED),
        Err(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
