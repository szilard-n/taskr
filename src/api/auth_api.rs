use actix_web::{HttpResponse, post};
use actix_web::web::{Data, Json};
use actix_web_httpauth::extractors::basic::BasicAuth;
use argonautica::{Hasher, Verifier};
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::SignWithKey;
use sha2::Sha256;

use crate::dto::create_user::CreateUser;
use crate::dto::token_claims::TokenClaims;
use crate::repository::user_repository::UserRepository;

#[post("/auth/sign-up")]
pub async fn sign_up(db: Data<UserRepository>, body: Json<CreateUser>) -> HttpResponse {
    let new_user: CreateUser = body.into_inner();
    let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET not provided");
    let mut hasher = Hasher::default();

    let password_hash = hasher
        .with_password(new_user.password)
        .with_secret_key(hash_secret)
        .hash()
        .unwrap();

    let user_details = db.create_user(new_user.email, password_hash).await;
    match user_details {
        Ok(_) => HttpResponse::Created().finish(),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[post("/auth/sign-in")]
pub async fn sign_in(db: Data<UserRepository>, credentials: BasicAuth) -> HttpResponse {
    let email = credentials.user_id();
    let request_password = match credentials.password() {
        Some(pwd) => pwd,
        None => return HttpResponse::Unauthorized().finish()
    };

    let user = match db.find_by_email(&String::from(email)).await {
        Ok(user_option) => {
            match user_option {
                Some(user) => user,
                None => return HttpResponse::Unauthorized().json("User not found")
            }
        }
        Err(e) => return HttpResponse::Unauthorized().json(format!("Something went wrong while signing in: {}", e))
    };

    let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET not provided");
    let mut verifier = Verifier::default();

    let password_valid = verifier
        .with_hash(user.password)
        .with_password(request_password)
        .with_secret_key(hash_secret)
        .verify()
        .unwrap();

    if password_valid {
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not provided!");
        let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).unwrap();

        let claims = TokenClaims { email: user.email };
        let token_str = claims.sign_with_key(&key).unwrap();

        HttpResponse::Ok().json(token_str)
    } else {
        HttpResponse::Unauthorized().json("Incorrect username or password")
    }
}