mod database;
mod models;
mod payloads;
mod schema;

use crate::database::{create_ticket, DataBase};
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
    let mut database = DataBase::new();

    if let Ok(payload) = serde_json::from_str::<TicketPayload>(&req_body) {
        return match create_ticket(&mut database.connection, payload.title, payload.body) {
            Ok(_) => HttpResponse::new(StatusCode::CREATED),
            Err(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        };
    }

    HttpResponse::new(StatusCode::BAD_REQUEST)
}

#[cfg(test)]
mod tests {
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
}
