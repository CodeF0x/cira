mod database;
mod errors;
mod models;
mod payloads;
mod schema;

use crate::database::{
    create_ticket, create_user, delete_ticket, edit_ticket, filter_tickets_in_database,
    get_all_tickets, DataBase,
};
use crate::errors::{
    ERROR_COULD_NOT_CREATE, ERROR_COULD_NOT_DELETE, ERROR_COULD_NOT_GET, ERROR_COULD_NOT_UPDATE,
    ERROR_INVALID_ID, ERROR_INVALID_JSON, ERROR_NOT_FOUND,
};
use crate::models::{NewUser, Ticket};
use crate::payloads::{FilterPayload, TicketPayload};
use actix_web::{delete, get, post, App, HttpRequest, HttpResponse, HttpServer, Responder};
use diesel::result::Error;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(create)
            .service(get_tickets)
            .service(delete)
            .service(edit)
            .service(sign_up)
            .service(filter_tickets)
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
            payload.assigned_user,
        ) {
            Ok(ticket) => HttpResponse::Created().json(ticket.to_ticket()),
            Err(_) => HttpResponse::InternalServerError().json(ERROR_COULD_NOT_CREATE),
        };
    }

    HttpResponse::BadRequest().json(ERROR_INVALID_JSON)
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
                .map(|sqlite_ticket| sqlite_ticket.to_ticket())
                .collect();

            HttpResponse::Ok().json(tickets)
        }
        Err(_) => HttpResponse::InternalServerError().json(ERROR_COULD_NOT_GET),
    }
}

#[post("/filter")]
async fn filter_tickets(req_body: String) -> impl Responder {
    if let Ok(filter) = serde_json::from_str::<FilterPayload>(&req_body) {
        let mut database = DataBase::new();

        return HttpResponse::Ok()
            .json(filter_tickets_in_database(&mut database.connection, filter));
    }

    HttpResponse::BadRequest().json(ERROR_INVALID_JSON)
}

#[post("/tickets/{id}")]
async fn edit(req: HttpRequest, req_body: String) -> impl Responder {
    let ticket_id: i32 = req
        .match_info()
        .get("id")
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    if ticket_id < 1 {
        return HttpResponse::BadRequest().json(ERROR_INVALID_ID);
    }

    if let Ok(ticket_payload) = serde_json::from_str::<TicketPayload>(&req_body) {
        let mut database = DataBase::new();

        return match edit_ticket(&mut database.connection, ticket_payload, ticket_id) {
            Ok(updated_ticket) => HttpResponse::Ok().json(updated_ticket.to_ticket()),
            Err(err) => match err {
                Error::NotFound => {
                    HttpResponse::NotFound().json(format!("{} {}", ERROR_NOT_FOUND, ticket_id))
                }
                _ => HttpResponse::InternalServerError()
                    .json(format!("{} {}", ERROR_COULD_NOT_UPDATE, ticket_id)),
            },
        };
    }

    HttpResponse::BadRequest().json(ERROR_INVALID_JSON)
}

#[delete("/tickets/{id}")]
async fn delete(req: HttpRequest) -> impl Responder {
    let ticket_id: i32 = req
        .match_info()
        .get("id")
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    if ticket_id < 1 {
        return HttpResponse::BadRequest().json(ERROR_INVALID_ID);
    }

    let mut database = DataBase::new();

    match delete_ticket(&mut database.connection, ticket_id) {
        Ok(sqlite_ticket) => HttpResponse::Ok().json(sqlite_ticket.to_ticket()),
        Err(err) => match err {
            Error::NotFound => {
                HttpResponse::NotFound().json(format!("{} {}", ERROR_NOT_FOUND, ticket_id))
            }
            _ => HttpResponse::InternalServerError()
                .json(format!("{} {}", ERROR_COULD_NOT_DELETE, ticket_id)),
        },
    }
}

#[post("/users")]
async fn sign_up(req_body: String) -> impl Responder {
    if let Ok(user_payload) = serde_json::from_str::<NewUser>(&req_body) {
        let mut database = DataBase::new();

        return match create_user(&mut database.connection, user_payload) {
            Ok(new_user) => HttpResponse::Created().json(new_user),
            Err(_) => HttpResponse::InternalServerError().json("Could not create new user"),
        };
    }

    HttpResponse::BadRequest().json(ERROR_INVALID_JSON)
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
        use actix_web::http::StatusCode;
        use serial_test::parallel;

        #[actix_web::test]
        #[parallel]
        async fn test_bad_request() {
            let app = test::init_service(App::new().service(create)).await;
            let req = TestRequest::post()
                .uri("/tickets")
                .data("{ \"title\": \"test title\" }")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
            assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);
        }

        #[actix_web::test]
        // serial is needed because sqlite does not support parallel write access -> run everything serially
        #[serial]
        async fn test_create_ticket() {
            setup_database();

            let ticket_payload =
                "{ \"title\": \"test title\", \"body\": \"test body\", \"labels\": [\"Bug\"], \"assigned_user\": 1 }";

            let app = test::init_service(App::new().service(create)).await;
            let req = TestRequest::post()
                .uri("/tickets")
                .set_payload(ticket_payload)
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
        use crate::models::Ticket;
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
        use actix_web::http::StatusCode;
        use serial_test::parallel;

        #[actix_web::test]
        #[serial]
        async fn test_delete_ticket() {
            setup_database();

            let app = test::init_service(App::new().service(delete)).await;
            let req = TestRequest::delete().uri("/tickets/1").to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_success());
            assert_eq!(response.status().as_u16(), StatusCode::OK);
        }

        #[actix_web::test]
        #[serial]
        async fn test_ticket_not_found() {
            setup_database();

            let app = test::init_service(App::new().service(delete)).await;
            let req = TestRequest::delete().uri("/tickets/999").to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
            assert_eq!(response.status().as_u16(), StatusCode::NOT_FOUND);
        }

        #[actix_web::test]
        #[parallel]
        async fn test_negative_id() {
            let app = test::init_service(App::new().service(delete)).await;
            let req = TestRequest::delete().uri("/tickets/-1").to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
            assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);
        }
    }

    mod edit_ticket {
        use crate::database::setup_database;
        use crate::edit;
        use actix_web::test::TestRequest;
        use actix_web::{test, App};
        use serial_test::{parallel, serial};

        #[actix_web::test]
        #[parallel]
        async fn test_invalid_id() {
            let app = test::init_service(App::new().service(edit)).await;
            let req = TestRequest::post()
                .uri("/tickets/invalid")
                .set_payload("")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
        }

        #[actix_web::test]
        #[parallel]
        async fn test_negative_id() {
            let app = test::init_service(App::new().service(edit)).await;
            let req = TestRequest::delete()
                .uri("/tickets/-1")
                .set_payload("{}")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
        }

        #[actix_web::test]
        #[parallel]
        async fn test_bad_payload() {
            let app = test::init_service(App::new().service(edit)).await;
            let req = TestRequest::post()
                .uri("/tickets/999")
                .set_payload("{ \"body\": \"Test\", \"title\": \"Test\" }")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
        }

        #[actix_web::test]
        #[serial]
        async fn test_not_found() {
            setup_database();

            let app = test::init_service(App::new().service(edit)).await;
            let req = TestRequest::post()
                .uri("/tickets/3")
                .set_payload("{}")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
        }

        #[actix_web::test]
        #[serial]
        async fn test_updated() {
            setup_database();

            let app = test::init_service(App::new().service(edit)).await;
            let req = TestRequest::post()
                .uri("/tickets/1")
                .set_payload(
                    "{ \"body\": \"Test\", \"title\": \"Test\", \"labels\": [\"Feature\"] }",
                )
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_success());
        }
    }

    mod test_sign_up {
        use super::*;
        use crate::database::setup_database;
        use crate::models::DataBaseUser;
        use crate::sign_up;
        use serial_test::parallel;

        #[actix_web::test]
        #[parallel]
        async fn test_bad_request() {
            let app = test::init_service(App::new().service(sign_up)).await;
            let req = TestRequest::post()
                .uri("/users")
                .set_payload(
                    "{ \"password\": \"123\", \"display_name\": \"User\", \"email\": 123 }",
                )
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
        }

        #[actix_web::test]
        #[serial]
        async fn test_sign_up() {
            setup_database();

            let email = "test@example.com";
            let display_name = "User";
            let password = "123";

            let payload = format!(
                "{{ \"password\": \"{}\", \"display_name\": \"{}\", \"email\": \"{}\" }}",
                password, display_name, email
            );

            let app = test::init_service(App::new().service(sign_up)).await;
            let req = TestRequest::post()
                .uri("/users")
                .set_payload(payload)
                .to_request();

            let response: DataBaseUser = test::call_and_read_body_json(&app, req).await;

            assert_eq!(response.email, email);
            assert_ne!(response.password, password);
            assert_eq!(response.display_name, display_name);
        }
    }
}
