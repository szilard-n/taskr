use actix_web::{Error, HttpMessage, HttpResponse};
use actix_web::dev::ServiceRequest;
use actix_web::web::{Data, Json};
use actix_web_httpauth::extractors::{AuthenticationError, bearer};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::VerifyWithKey;
use serde::de::DeserializeOwned;
use sha2::Sha256;
use validator::Validate;

use crate::dto::token_claims::TokenClaims;
use crate::model::user_model::User;
use crate::repository::user_repository::UserRepository;

pub async fn jwt_validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not provided!");
    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).unwrap();
    let bearer_token = credentials.token();
    let config = req.app_data::<bearer::Config>().cloned().unwrap_or_default().scope("");

    let token_claims = match bearer_token.verify_with_key(&key) {
        Ok(claims) => claims,
        Err(_) => return Err((AuthenticationError::from(config).into(), req)),
    };

    let db = req.app_data::<Data<UserRepository>>().unwrap();
    let user = match find_user_by_claims(&db, &token_claims).await {
        Ok(Some(user)) => user,
        _ => return Err((AuthenticationError::from(config).into(), req)),
    };

    req.extensions_mut().insert(user);
    Ok(req)
}

async fn find_user_by_claims(db: &Data<UserRepository>, claims: &TokenClaims) -> Result<Option<User>, ()> {
    match db.find_by_email(&claims.email).await {
        Ok(user_option) => Ok(user_option),
        Err(_) => Err(()),
    }
}

pub async fn validate_request_body<T>(body: Json<T>) -> Result<T, HttpResponse>
    where T: DeserializeOwned + Validate + 'static {
    let value = body.into_inner();
    match value.validate() {
        Ok(_) => Ok(value),
        Err(e) => Err(HttpResponse::BadRequest().body(format!("{}", e)))
    }
}