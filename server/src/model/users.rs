use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;
use crate::util::jwt;
use crate::{schema::users, Database};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use chrono::NaiveDateTime;
use diesel::{insert_into, QueryDsl};

use protocol::protocol::Credentials;
use protocol::protocol::LobbyInfo;
use protocol::protocol::LoginResponse;
use protocol::protocol::Protocol;
use protocol::protocol::UserData;
use rand_core::OsRng;
use rocket::http::Status;
use rocket::request;
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::serde::json::Json;
use rocket::Request;

use super::lobbys::LobbyWithUsers;

#[derive(Queryable, Clone, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub currency: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
    Other,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r User {
    type Error = ApiKeyError;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let user = req
            .local_cache_async(async {
                match req.headers().get_one("x-api-key") {
                    None => Err(ApiKeyError::Missing),
                    Some(key) => {
                        trace!("Authenticating based on jwt");
                        match jwt::validate(key) {
                            Ok(token) => {
                                trace!("Token is valid");
                                if let Some(db) = req.guard::<Database>().await.succeeded() {
                                    let user = db
                                        .run(move |con| {
                                            users::table
                                                .filter(users::username.eq(token.claims.sub))
                                                .first::<User>(con)
                                        })
                                        .await
                                        .expect("Failed to retrieve user from db");
                                    trace!("Setting user {:?}", user);

                                    return Ok(user);
                                }
                                Err(ApiKeyError::Other)
                            }
                            Err(err) => {
                                warn!("Error during login: {}", err);
                                Err(ApiKeyError::Invalid)
                            }
                        }
                    }
                }
            })
            .await;
        match user {
            Ok(user) => Outcome::Success(&user),
            Err(e) => Outcome::Failure((Status::Unauthorized, e.clone())),
        }
    }
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub salt: String,
}

impl NewUser {
    fn from_credentials(cred: &Credentials) -> Result<NewUser, ()> {
        let salt = SaltString::generate(&mut OsRng);
        let username: String = cred.username.clone();

        // Hash password to PHC string ($argon2id$v=19$...)
        if let Ok(password_hash) = hash_password(cred, &salt) {
            return Ok(NewUser {
                username,
                password: password_hash,
                salt: salt.to_string(),
            });
        }
        Err(())
    }
}

fn hash_password(cred: &Credentials, salt: &SaltString) -> Result<String, ()> {
    // Argon2 with default params (Argon2id v19)
    // Hash password to PHC string ($argon2id$v=19$...)
    if let Ok(hash) = Argon2::default().hash_password(cred.password.as_bytes(), &salt) {
        return Ok(hash.to_string());
    }

    Err(())
}

#[put("/users", data = "<creds>")]
pub async fn register(creds: Json<Credentials>, db: Database) -> Json<Protocol> {
    let new_user = NewUser::from_credentials(&creds).expect("Failed to hash password");

    // TODO: return new user
    db.run(|con| insert_into(users::table).values(new_user).execute(con))
        .await
        .expect("Failed to create user");

    Json(Protocol::LOGIN_RESPONSE(LoginResponse {
        key: jwt::generate(&creds.username),
        user: UserData {
            username: creds.username.clone(),
            currency: 0,
            lobby: None,
        },
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

    Json(Protocol::LOGIN_RESPONSE(LoginResponse {
        key: jwt::generate(&user.username),
        user: UserData {
            username: user.username,
            currency: user.currency,
            lobby: None,
        },
    }))
}

#[get("/users/@me")]
pub fn me(user: &User, lobby: Option<LobbyWithUsers>) -> Json<Protocol> {
    Json(Protocol::USER_RESPONSE(UserData {
        username: user.username.to_string(),
        currency: user.currency,
        lobby: lobby.and_then(|l| {
            Some(LobbyInfo {
                name: l.lobby.name,
                users: l.users.into_iter().map(|u| u.into()).collect(),
            })
        }),
    }))
}
