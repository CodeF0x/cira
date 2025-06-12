use crate::database::{session_in_db, DataBase};
use crate::models::TokenClaims;
use actix_web::dev::ServiceRequest;
use actix_web::{Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::{bearer, AuthenticationError};
use dotenvy::dotenv;
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::VerifyWithKey;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    dotenv().ok();

    let jwt_secret: String = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set!");
    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).unwrap();
    let token_string = credentials.token();
    let mut database = DataBase::new();

    // in theory, when this check succeeds, is a actual validation even necessary?
    if !session_in_db(token_string.to_string(), &mut database.connection) {
        let config = req
            .app_data::<bearer::Config>()
            .cloned()
            .unwrap_or_default()
            .scope("");

        return Err((AuthenticationError::from(config).into(), req));
    }

    let claims: Result<TokenClaims, &str> = token_string
        .verify_with_key(&key)
        .map_err(|_| "Invalid token");

    match claims {
        Ok(value) => {
            if value.exp
                < SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            {
                let config = Config::default().scope("");

                return Err((AuthenticationError::from(config).into(), req));
            }

            req.extensions_mut().insert(value);
            Ok(req)
        }
        Err(_) => {
            let config = Config::default().scope("");

            Err((AuthenticationError::from(config).into(), req))
        }
    }
}
