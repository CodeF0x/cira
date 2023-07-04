mod database;
mod models;
mod payloads;
mod schema;

use crate::database::{create_ticket, get_all_tickets, DataBase};
use crate::payloads::TicketPayload;
use actix_web::http::StatusCode;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    HttpServer::new(|| App::new().service(create).service(get_tickets))
        .bind(("localhost", 8080))?
        .run()
        .await
}

#[post("/tickets")]
async fn create(req_body: String) -> impl Responder {
    let mut database = DataBase::new();

    // todo: refactor this to use a JSON extractor after figuring out on how to test this
    if let Ok(payload) = serde_json::from_str::<TicketPayload>(&req_body) {
        return match create_ticket(&mut database.connection, payload.title, payload.body) {
            Ok(_) => HttpResponse::new(StatusCode::CREATED),
            Err(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        };
    }

    HttpResponse::new(StatusCode::BAD_REQUEST)
}

#[get("/tickets")]
async fn get_tickets() -> impl Responder {
    let mut database = DataBase::new();

    match get_all_tickets(&mut database.connection) {
        Ok(tickets) => HttpResponse::Ok().json(tickets),
        Err(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[cfg(test)]
mod tests {
    // todo: refactor so that tests do not depend on each other, e. g. by always resetting the test db to a default value after a test

    use actix_web::test::TestRequest;
    use actix_web::{test, App};

    mod create_ticket {
        use super::*;
        use crate::create;

        #[actix_web::test]
        async fn test_bad_request() {
            let app = test::init_service(App::new().service(create)).await;
            let req = TestRequest::post()
                .uri("/tickets")
                .data("{ \"title\": \"test title\" }")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
        }

        #[actix_web::test]
        async fn test_create_ticket() {
            let app = test::init_service(App::new().service(create)).await;
            let req = TestRequest::post()
                .uri("/tickets")
                .set_payload("{ \"title\": \"test title\", \"body\": \"test body\" }")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_success());
        }
    }

    mod get_tickets {
        use super::*;
        use crate::get_tickets;
        use crate::models::Ticket;
        use actix_web::test;

        #[actix_web::test]
        async fn test_get_tickets() {
            let app = test::init_service(App::new().service(get_tickets)).await;
            let req = TestRequest::get().uri("/tickets").to_request();

            let response: Vec<Ticket> = test::call_and_read_body_json(&app, req).await;

            assert!(response.len() > 0);
        }
    }
}
