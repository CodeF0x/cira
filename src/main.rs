mod database;
mod filters;
mod middleware;
mod models;
mod payloads;
mod schema;
mod status_messages;
mod test_helpers;

use crate::database::{
    create_ticket, create_user, delete_ticket, edit_ticket, filter_tickets_in_database,
    get_all_tickets, get_user_by_email, remove_session_from_db, write_session_to_db, DataBase,
};
use crate::middleware::validator;
use crate::models::{NewSession, NewUser, Ticket, TokenClaims};
use crate::payloads::{FilterPayload, LoginPayload, TicketPayload};
use crate::status_messages::{
    CANNOT_LOGOUT, ERROR_COULD_NOT_CREATE_TICKET, ERROR_COULD_NOT_CREATE_USER,
    ERROR_COULD_NOT_DELETE, ERROR_COULD_NOT_GET, ERROR_COULD_NOT_UPDATE, ERROR_INCORRECT_PASSWORD,
    ERROR_INVALID_ID, ERROR_INVALID_JSON, ERROR_NOT_FOUND, ERROR_NOT_LOGGED_IN,
    ERROR_NO_USER_FOUND, SUCCESS_LOGIN, SUCCESS_LOGOUT,
};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::{delete, get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::middleware::HttpAuthentication;
use argonautica::Verifier;
use diesel::result::Error;
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::SignWithKey;
use sha2::Sha256;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(validator);

        App::new().service(sign_up).service(login).service(
            web::scope("")
                .wrap(bearer_middleware)
                .service(create)
                .service(get_tickets)
                .service(delete)
                .service(edit)
                .service(filter_tickets)
                .service(logout),
        )
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
            Err(_) => HttpResponse::InternalServerError().json(ERROR_COULD_NOT_CREATE_TICKET),
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

        return match filter_tickets_in_database(&mut database.connection, filter) {
            Ok(tickets) => HttpResponse::Ok().json(tickets),
            Err(_) => HttpResponse::InternalServerError().json(ERROR_COULD_NOT_GET),
        };
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

#[post("/sign_up")]
async fn sign_up(req_body: String) -> impl Responder {
    if let Ok(user_payload) = serde_json::from_str::<NewUser>(&req_body) {
        let mut database = DataBase::new();

        return match create_user(&mut database.connection, user_payload) {
            Ok(new_user) => HttpResponse::Created().json(new_user),
            Err(_) => HttpResponse::InternalServerError().json(ERROR_COULD_NOT_CREATE_USER),
        };
    }

    HttpResponse::BadRequest().json(ERROR_INVALID_JSON)
}

#[post("/log_out")]
async fn logout(bearer: BearerAuth) -> impl Responder {
    let mut database = DataBase::new();

    return match remove_session_from_db(bearer.token().to_string(), &mut database.connection) {
        Ok(rows_affected) => {
            return match rows_affected {
                0 => HttpResponse::NotFound().json(ERROR_NOT_LOGGED_IN),
                _ => {
                    let bearer_cookie = Cookie::build("cira-bearer-token", "")
                        .http_only(true)
                        .max_age(Duration::new(-1, 0))
                        .finish();
                    HttpResponse::Ok()
                        .cookie(bearer_cookie)
                        .json(SUCCESS_LOGOUT)
                }
            }
        }
        Err(_) => HttpResponse::InternalServerError().json(CANNOT_LOGOUT),
    };
}

#[post("/login")]
async fn login(req_body: String) -> impl Responder {
    if let Ok(login_payload) = serde_json::from_str::<LoginPayload>(&req_body) {
        let mut database = DataBase::new();
        let jwt_secret: Hmac<Sha256> = Hmac::new_from_slice(
            std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set!")
                .as_bytes(),
        )
        .unwrap();

        return match get_user_by_email(login_payload.email, &mut database.connection) {
            Ok(database_user) => {
                let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET must be set!");
                let mut verifier = Verifier::default();
                let is_valid = verifier
                    .with_hash(database_user.password)
                    .with_password(login_payload.password)
                    .with_secret_key(hash_secret)
                    .verify()
                    .unwrap();

                if is_valid {
                    let claims = TokenClaims {
                        id: database_user.id,
                    };
                    let token_str = claims.sign_with_key(&jwt_secret).unwrap();

                    write_session_to_db(
                        NewSession {
                            token: token_str.clone(),
                        },
                        &mut database.connection,
                    );

                    let bearer_cookie = Cookie::build("cira-bearer-token", token_str)
                        .http_only(true)
                        .finish();
                    HttpResponse::Ok().cookie(bearer_cookie).json(SUCCESS_LOGIN)
                } else {
                    HttpResponse::Unauthorized().json(ERROR_INCORRECT_PASSWORD)
                }
            }
            Err(err) => {
                return match err {
                    Error::NotFound => HttpResponse::NotFound().json(ERROR_NO_USER_FOUND),
                    _ => HttpResponse::InternalServerError().json(ERROR_COULD_NOT_CREATE_USER),
                }
            }
        };
    }

    HttpResponse::BadRequest().json(ERROR_INVALID_JSON)
}

/*
* To fully understand the tests and the test data,
* have a look at the setup_database function in test_helpers.rs.
 */
#[cfg(test)]
mod tests {
    use crate::test_helpers::helpers::{reset_database, setup_database};
    use actix_web::test::TestRequest;
    use actix_web::{test, App};
    use serial_test::serial;

    mod create_ticket {
        use super::*;
        use crate::create;
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
                "{ \"title\": \"test title\", \"body\": \"test body\", \"labels\": [\"Bug\"], \"assigned_user\": 1, \"status\": \"Open\" }";

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

            assert!(!response.is_empty());
        }

        #[actix_web::test]
        #[serial]
        async fn test_get_tickets_none() {
            reset_database();

            let app = test::init_service(App::new().service(get_tickets)).await;
            let req = TestRequest::get().uri("/tickets").to_request();

            let response: Vec<Ticket> = test::call_and_read_body_json(&app, req).await;

            assert!(response.is_empty());
        }
    }

    mod delete_ticket {
        use super::*;
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
        use super::*;
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
                    "{ \"body\": \"Test\", \"title\": \"Test\", \"labels\": [\"Feature\"], \"status\": \"Open\" }",
                )
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_success());
        }
    }

    mod test_sign_up {
        use super::*;
        use crate::models::DataBaseUser;
        use crate::sign_up;
        use serial_test::parallel;

        #[actix_web::test]
        #[parallel]
        async fn test_bad_request() {
            let app = test::init_service(App::new().service(sign_up)).await;
            let req = TestRequest::post()
                .uri("/sign_up")
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
                .uri("/sign_up")
                .set_payload(payload)
                .to_request();

            let response: DataBaseUser = test::call_and_read_body_json(&app, req).await;

            assert_eq!(response.email, email);
            assert_ne!(response.password, password);
            assert_eq!(response.display_name, display_name);
        }
    }

    mod test_filter {
        use super::*;
        use crate::filter_tickets;
        use crate::models::Ticket;
        use serial_test::parallel;

        #[actix_web::test]
        #[parallel]
        async fn test_bad_request() {
            let app = test::init_service(App::new().service(filter_tickets)).await;
            let req = TestRequest::post()
                .uri("/filter")
                .set_payload("{ \"labels\": null, }") // mind the trailing comma
                .to_request();

            let response = test::call_service(&app, req).await;

            assert!(response.status().is_client_error());
        }

        #[actix_web::test]
        #[serial]
        async fn test_filter_by_labels() {
            setup_database();

            let app = test::init_service(App::new().service(filter_tickets)).await;
            let req = TestRequest::post()
                .uri("/filter")
                .set_payload("{ \"labels\": [\"InProgress\", \"Bug\"] }")
                .to_request();

            let response: Vec<Ticket> = test::call_and_read_body_json(&app, req).await;

            assert_eq!(response.len(), 1);
        }

        #[actix_web::test]
        #[serial]
        async fn test_filter_by_assigned_user() {
            setup_database();

            let app = test::init_service(App::new().service(filter_tickets)).await;
            let req = TestRequest::post()
                .uri("/filter")
                .set_payload("{ \"assigned_user\": 1 }")
                .to_request();

            let response: Vec<Ticket> = test::call_and_read_body_json(&app, req).await;

            assert_eq!(response.len(), 1);
        }

        #[actix_web::test]
        #[serial]
        async fn test_filter_by_title() {
            setup_database();

            let app = test::init_service(App::new().service(filter_tickets)).await;
            let req = TestRequest::post()
                .uri("/filter")
                .set_payload("{ \"title\": \"Test\" }")
                .to_request();

            let response: Vec<Ticket> = test::call_and_read_body_json(&app, req).await;

            assert_eq!(response.len(), 1);
        }

        #[actix_web::test]
        #[serial]
        async fn test_filter_by_status() {
            setup_database();

            let app = test::init_service(App::new().service(filter_tickets)).await;
            let req = TestRequest::post()
                .uri("/filter")
                .set_payload("{ \"status\": \"Open\" }")
                .to_request();

            let response: Vec<Ticket> = test::call_and_read_body_json(&app, req).await;

            assert_eq!(response.len(), 1);
        }

        #[actix_web::test]
        #[serial]
        async fn test_return_all_when_filter_is_empty() {
            setup_database();

            let app = test::init_service(App::new().service(filter_tickets)).await;
            let req = TestRequest::post()
                .uri("/filter")
                .set_payload("{}")
                .to_request();

            let response: Vec<Ticket> = test::call_and_read_body_json(&app, req).await;

            assert_eq!(response.len(), 1);
        }

        #[actix_web::test]
        #[serial]
        async fn test_return_nothing_if_nothing_matches() {
            setup_database();

            let app = test::init_service(App::new().service(filter_tickets)).await;
            let req = TestRequest::post()
                .uri("/filter")
                .set_payload("{ \"assigned_user\": 999 }")
                .to_request();

            let response: Vec<Ticket> = test::call_and_read_body_json(&app, req).await;

            assert_eq!(response.len(), 0);
        }
    }

    mod test_login {
        use super::*;
        use crate::database::DataBase;
        use crate::login;
        use crate::models::DatabaseSession;
        use crate::schema::sessions::dsl::sessions;
        use crate::test_helpers::helpers::setup_database;
        use actix_web::http::StatusCode;
        use actix_web::test;
        use diesel::RunQueryDsl;
        use serial_test::parallel;

        #[actix_web::test]
        #[serial]
        async fn test_login() {
            setup_database();

            let app = test::init_service(App::new().service(login)).await;
            let req = TestRequest::post()
                .uri("/login")
                .set_payload("{ \"email\": \"test@example.com\", \"password\": \"123\" }")
                .to_request();

            let response = test::call_service(&app, req).await;
            assert!(response
                .headers()
                .get("set-cookie")
                .unwrap()
                .to_str()
                .unwrap()
                .contains("cira-bearer-token"));

            let mut db = DataBase::new();

            let tokens_in_db: Vec<DatabaseSession> = sessions.load(&mut db.connection).unwrap();

            assert_ne!(tokens_in_db.first().unwrap().token, "".to_string());
        }

        #[actix_web::test]
        #[serial]
        async fn test_wrong_password() {
            setup_database();

            let app = test::init_service(App::new().service(login)).await;
            let req = TestRequest::post()
                .uri("/login")
                .set_payload(
                    "{ \"email\": \"test@example.com\", \"password\": \"wrong-password\" }",
                )
                .to_request();

            let response = test::call_service(&app, req).await;

            assert_eq!(response.status().as_u16(), StatusCode::UNAUTHORIZED);
        }

        #[actix_web::test]
        #[serial]
        async fn test_email_not_found() {
            setup_database();

            let app = test::init_service(App::new().service(login)).await;
            let req = TestRequest::post()
                .uri("/login")
                .set_payload("{ \"email\": \"doesnotexist@test.de\", \"password\": \"123\" }")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert_eq!(response.status().as_u16(), StatusCode::NOT_FOUND);
        }

        #[actix_web::test]
        #[parallel]
        async fn test_invalid_payload() {
            let app = test::init_service(App::new().service(login)).await;
            let req = TestRequest::post()
                .uri("/login")
                .set_payload("{}")
                .to_request();

            let response = test::call_service(&app, req).await;

            assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);
        }
    }

    mod test_logout {
        use crate::database::DataBase;
        use crate::logout;
        use crate::models::{DatabaseSession, NewSession};
        use crate::schema::sessions::dsl::sessions;
        use crate::test_helpers::helpers::setup_database;
        use actix_web::http::StatusCode;
        use actix_web::test::TestRequest;
        use actix_web::{test, App};
        use diesel::RunQueryDsl;
        use serial_test::serial;

        #[actix_web::test]
        #[serial]
        async fn test_logout() {
            setup_database();

            let mut db = DataBase::new();
            diesel::insert_into(sessions)
                .values(NewSession {
                    token: "123".to_string(),
                })
                .execute(&mut db.connection)
                .unwrap();

            let app = test::init_service(App::new().service(logout)).await;
            let req = TestRequest::post()
                .uri("/log_out")
                .insert_header(("Authorization", "Bearer 123"))
                .to_request();

            let response = test::call_service(&app, req).await;

            let active_sessions: Vec<DatabaseSession> = sessions.load(&mut db.connection).unwrap();

            assert_eq!(response.status().as_u16(), StatusCode::OK);
            assert_eq!(active_sessions.len(), 0);
        }

        #[actix_web::test]
        #[serial]
        async fn test_no_session_in_db() {
            setup_database();

            let mut db = DataBase::new();
            diesel::insert_into(sessions)
                .values(NewSession {
                    token: "123".to_string(),
                })
                .execute(&mut db.connection)
                .unwrap();

            let app = test::init_service(App::new().service(logout)).await;
            let req = TestRequest::post()
                .uri("/log_out")
                .insert_header(("Authorization", "Bearer 404"))
                .to_request();

            let response = test::call_service(&app, req).await;

            assert_eq!(response.status().as_u16(), StatusCode::NOT_FOUND);
        }

        // 401 when bearer token is missing is handled by lib
    }

    mod test_middleware {
        use super::*;
        use crate::database::DataBase;
        use crate::get_tickets;
        use crate::middleware::validator;
        use crate::models::NewSession;
        use crate::schema::sessions::dsl::sessions;
        use actix_web::http::StatusCode;
        use actix_web::{test, web};
        use actix_web_httpauth::middleware::HttpAuthentication;
        use diesel::RunQueryDsl;
        use serial_test::serial;

        #[actix_web::test]
        #[serial]
        async fn test_middleware() {
            setup_database();

            // bearer for "123"
            let bearer_token =
                "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6MX0.oi92tHHWj5HdQO8Hd9vIYD6suTWosoiBnpdRBIcNGpM";

            let mut db = DataBase::new();
            diesel::insert_into(sessions)
                .values(NewSession {
                    token: bearer_token.to_string(),
                })
                .execute(&mut db.connection)
                .unwrap();

            let app = test::init_service(
                App::new().service(
                    web::scope("")
                        .wrap(HttpAuthentication::bearer(validator))
                        .service(get_tickets),
                ),
            )
            .await;
            let req = TestRequest::get()
                .uri("/tickets")
                .insert_header(("Authorization", format!("Bearer {bearer_token}")))
                .to_request();

            let response = test::call_service(&app, req).await;

            assert_eq!(response.status().as_u16(), StatusCode::OK);
        }

        #[actix_web::test]
        #[serial]
        async fn test_session_not_in_db() {
            setup_database();

            let mut db = DataBase::new();
            diesel::insert_into(sessions)
                .values(NewSession {
                    token: "123".to_string(),
                })
                .execute(&mut db.connection)
                .unwrap();

            let app = test::init_service(
                App::new().service(
                    web::scope("")
                        .wrap(HttpAuthentication::bearer(validator))
                        .service(get_tickets),
                ),
            )
            .await;
            let req = TestRequest::get()
                .uri("/tickets")
                .insert_header(("Authorization", "Bearer 404"))
                .to_request();

            let response = test::call_service(&app, req).await;

            assert_eq!(response.status().as_u16(), StatusCode::UNAUTHORIZED);
        }

        #[actix_web::test]
        #[serial]
        async fn test_wrong_bearer_token() {
            setup_database();

            let mut db = DataBase::new();
            diesel::insert_into(sessions)
                .values(NewSession {
                    // actual working token, but removed last few characters
                    token: "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6MX0.oi92tHHWj5HdQO8Hd9vIYD6suTWosoiBnpd"
                        .to_string(),
                })
                .execute(&mut db.connection)
                .unwrap();

            let app = test::init_service(
                App::new().service(
                    web::scope("")
                        .wrap(HttpAuthentication::bearer(validator))
                        .service(get_tickets),
                ),
            )
            .await;
            let req = TestRequest::get().uri("/tickets").insert_header(("Authorization", "Bearer eyJhbGciOiJIUzI1NiJ9.eyJpZCI6MX0.oi92tHHWj5HdQO8Hd9vIYD6suTWosoiBnpdRBIcNGpM")).to_request();

            let response = test::call_service(&app, req).await;

            assert_eq!(response.status().as_u16(), StatusCode::UNAUTHORIZED);
        }
    }
}
