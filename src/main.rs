mod database;
mod models;
mod payloads;
mod schema;

use crate::database::{create_ticket, delete_ticket, get_all_tickets, DataBase};
use crate::models::Ticket;
use crate::payloads::{TicketPayload, TicketToDelete};
use actix_web::http::StatusCode;
use actix_web::{delete, get, post, App, HttpResponse, HttpServer, Responder};
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(create)
            .service(get_tickets)
            .service(delete)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}

#[post("/tickets")]
async fn create(req_body: String) -> impl Responder {
    let mut database = DataBase::new();

    // todo: refactor this to use a JSON extractor after figuring out on how to test this
    if let Ok(payload) = serde_json::from_str::<TicketPayload>(&req_body) {
        return match create_ticket(
            &mut database.connection,
            payload.title,
            payload.body,
            payload.labels,
        ) {
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
        // sqlite does not support arrays, to to return proper json, need to parse the labels string into actual json
        // without this, labels property would be an escaped string, not an actual json array
        Ok(tickets) => {
            let tickets: Vec<Ticket> = tickets
                .iter()
                .map(|ticket| {
                    let labels = serde_json::from_str(&ticket.labels)
                        .expect("Could not parse labels array. This should not be possible.");
                    return Ticket {
                        id: ticket.id.clone(),
                        title: ticket.title.to_string(),
                        body: ticket.body.to_string(),
                        created: ticket.created.clone(),
                        last_modified: ticket.last_modified.clone(),
                        labels,
                    };
                })
                .collect();

            return HttpResponse::Ok().json(tickets);
        }
        Err(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[delete("/tickets")]
async fn delete(req_body: String) -> impl Responder {
    let mut database = DataBase::new();

    // todo: refactor once understood how to test with JSON extractors
    if let Ok(ticket) = serde_json::from_str::<TicketToDelete>(&req_body) {
        return match delete_ticket(&mut database.connection, ticket.id) {
            Ok(rows_affected) => {
                if rows_affected == 0 {
                    return HttpResponse::new(StatusCode::NOT_FOUND);
                }
                HttpResponse::new(StatusCode::OK)
            },
            Err(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        };
    }

    HttpResponse::new(StatusCode::BAD_REQUEST)
}

#[cfg(test)]
mod tests {
    use actix_web::test::TestRequest;
    use actix_web::{test, App};
    use serial_test::serial;

    mod create_ticket {
        use super::*;
        use crate::create;
        use crate::database::setup_database;

        #[actix_web::test]
        // serial is needed because sqlite does not support parallel write access -> run everything serially
        #[serial]
        async fn test_bad_request() {
            setup_database();

            let app = test::init_service(App::new().service(create)).await;
            let req = TestRequest::post()
                .uri("/tickets")
                .data("{ \"title\": \"test title\" }")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
            assert_eq!(response.status().as_u16(), 400);
        }

        #[actix_web::test]
        #[serial]
        async fn test_create_ticket() {
            setup_database();

            let app = test::init_service(App::new().service(create)).await;
            let req = TestRequest::post()
                .uri("/tickets")
                .set_payload(
                    "{ \"title\": \"test title\", \"body\": \"test body\", \"labels\": [] }",
                )
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_success());
            assert_eq!(response.status().as_u16(), 201);
        }
    }

    mod get_tickets {
        use super::*;
        use crate::database::setup_database;
        use crate::get_tickets;
        use crate::models::{Ticket};
        use actix_web::test;

        #[actix_web::test]
        #[serial]
        async fn test_get_tickets() {
            setup_database();

            let app = test::init_service(App::new().service(get_tickets)).await;
            let req = TestRequest::get().uri("/tickets").to_request();

            let response: Vec<Ticket> = test::call_and_read_body_json(&app, req).await;

            assert!(response.len() > 0);
        }
    }

    mod delete_ticket {
        use super::*;
        use crate::database::setup_database;
        use crate::delete;

        #[actix_web::test]
        #[serial]
        async fn test_delete_ticket() {
            setup_database();

            let app = test::init_service(App::new().service(delete)).await;
            let req = TestRequest::delete()
                .uri("/tickets")
                .set_payload("{ \"id\": 1 }")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_success());
            assert_eq!(response.status().as_u16(), 200);
        }

        #[actix_web::test]
        #[serial]
        async fn test_ticket_not_found() {
            setup_database();

            let app = test::init_service(App::new().service(delete)).await;
            let req = TestRequest::delete()
                .uri("/tickets")
                .set_payload("{ \"id\": 999 }")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
            assert_eq!(response.status().as_u16(), 404);
        }

        #[actix_web::test]
        #[serial]
        async fn test_bad_request() {
            setup_database();

            let app = test::init_service(App::new().service(delete)).await;
            let req = TestRequest::delete()
                .uri("/tickets")
                .set_payload("{}")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
            assert_eq!(response.status().as_u16(), 400);
        }
    }
}
