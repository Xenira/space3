use std::error::Error;

use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;
use crate::util::jwt;
use crate::{schema::users, Database};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use chrono::{NaiveDateTime, Utc};
use diesel::{insert_into, QueryDsl};

use protocol::protocol::Credentials;
use protocol::protocol::Header;
use protocol::protocol::Protocol;
use protocol::protocol::UserData;
use rand_core::OsRng;
use rocket::http::Status;
use rocket::request;
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::serde::{json::Json, Deserialize};
use rocket::Request;

#[derive(Queryable)]
pub struct Lobby {
    pub id: i32,
    pub name: String,
    pub passphrase: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
    Other,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Lobby {
    type Error = ApiKeyError;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match req.headers().get_one("x-api-key") {
            None => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) => match jwt::validate(key) {
                Ok(token) => {
                    if let Some(db) = req.guard::<Database>().await.succeeded() {
                        let lobby = db
                            .run(move |con| {
                                lobbys::table
                                    .filter(users::username.eq(token.claims.sub))
                                    .first::<User>(con)
                            })
                            .await
                            .expect("Failed to retrieve user from db");
                        return Outcome::Success(lobby);
                    }
                    Outcome::Failure((Status::ServiceUnavailable, ApiKeyError::Other))
                }
                Err(_) => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
            },
        }
    }
}

#[put("/lobbys", data = "<creds>")]
pub async fn register(creds: Json<Credentials>, db: Database) -> Json<Protocol> {
    let new_user = NewUser::from_credentials(&creds).expect("Failed to hash password");

    db.run(|con| insert_into(users::table).values(new_user).execute(con))
        .await
        .expect("Failed to create user");

    Json(Protocol::SET_HEADER_RESPONSE(Header {
        name: "x-api-key".to_string(),
        value: jwt::generate(&creds.username),
    }))
}

#[post("/users", data = "<creds>")]
pub async fn login(creds: Json<Credentials>, db: Database) -> Json<Protocol> {
    let username = creds.username.clone();
    let user = db
        .run(move |con| {
            users::table
                .filter(users::username.eq(username))
                .first::<User>(con)
        })
        .await
        .expect("Failed to retrieve user from db");

    assert_eq!(
        hash_password(
            &creds,
            &SaltString::new(user.salt.as_str()).expect("User salt currupted")
        )
        .expect("Failed to verify login")
        .to_string(),
        user.password
    );

    Json(Protocol::SET_HEADER_RESPONSE(Header {
        name: "x-api-key".to_string(),
        value: jwt::generate(&user.username),
    }))
}

#[get("/users/@me")]
pub fn me(user: User) -> Json<Protocol> {
    Json(Protocol::USER_RESPONSE(UserData {
        username: user.username,
        currency: user.currency,
    }))
}
