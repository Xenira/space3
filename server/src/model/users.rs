use super::{lobbies::LobbyWithUsers, polling::ActivePolls};
use crate::{model::polling::Channel, schema::lobby_users, schema::users, Database, RunningGames};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use chrono::NaiveDateTime;
use diesel::{dsl::now, insert_into, prelude::*, update, ExpressionMethods, QueryDsl, RunQueryDsl};
use protocol::protocol::{Credentials, Error, LoginResponse, Protocol, UserData};
use rand_core::OsRng;
use rocket::{
    http::Status,
    log::private::debug,
    request::{self, FromRequest, Outcome},
    serde::json::Json,
    Request, State,
};
use std::fmt;
use uuid::Uuid;

#[derive(Identifiable, Queryable, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub display_name: Option<String>,
    pub currency: i32,
    pub tutorial: bool,
    pub session_token: Option<Uuid>,
    pub session_expires: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("display_name", &self.display_name)
            .field("password", &"[REDACTED]".to_string())
            .field("salt", &"[REDACTED]".to_string())
            .field("display_name", &self.display_name)
            .field("currency", &self.currency)
            .field("tutorial", &self.tutorial)
            .field("session_token", &"[REDACTED]".to_string())
            .field("session_expires", &self.session_expires)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
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
                match req.headers().get_one("x-api-key").map(|s| s.to_string()) {
                    None => Err(ApiKeyError::Missing),
                    Some(key) => {
                        if let Some(db) = req.guard::<Database>().await.succeeded() {
                            db.run(move |con| {
                                let user = users::table
                                    .filter(
                                        users::session_token
                                            .eq(Some(Uuid::parse_str(&key).unwrap()))
                                            .and(users::session_expires.gt(now)),
                                    )
                                    .first::<User>(con)
                                    .unwrap();

                                update(users::table)
                                    .filter(users::id.eq(user.id))
                                    .set((users::session_expires.eq(Some(
                                        chrono::Utc::now().naive_utc() + chrono::Duration::hours(1),
                                    )),))
                                    .execute(con)
                                    .unwrap();

                                trace!("Setting user {:?}", user);
                                Ok(user)
                            })
                            .await
                        } else {
                            Err(ApiKeyError::Other)
                        }
                    }
                }
            })
            .await;
        match user {
            Ok(user) => Outcome::Success(user),
            Err(e) => Outcome::Failure((Status::Unauthorized, e.clone())),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub salt: String,
    pub session_token: Option<Uuid>,
    pub session_expires: Option<NaiveDateTime>,
}

impl NewUser {
    fn from_credentials(cred: &Credentials, session_token: Uuid) -> Result<NewUser, ()> {
        let salt = SaltString::generate(&mut OsRng);
        let username: String = cred.username.clone();

        // Hash password to PHC string ($argon2id$v=19$...)
        if let Ok(password_hash) = hash_password(cred, &salt) {
            return Ok(NewUser {
                username,
                password: password_hash,
                salt: salt.to_string(),
                session_token: Some(session_token),
                session_expires: Some(chrono::Utc::now().naive_utc() + chrono::Duration::hours(1)),
            });
        }
        Err(())
    }
}

fn hash_password(cred: &Credentials, salt: &SaltString) -> Result<String, ()> {
    // Argon2 with default params (Argon2id v19)
    // Hash password to PHC string ($argon2id$v=19$...)
    if let Ok(hash) = Argon2::default().hash_password(cred.password.as_bytes(), salt) {
        return Ok(hash.to_string());
    }

    Err(())
}

#[put("/users", data = "<creds>")]
pub async fn register(creds: Json<Credentials>, db: Database) -> Json<Protocol> {
    let session_token = Uuid::new_v4();

    let new_user =
        NewUser::from_credentials(&creds, session_token).expect("Failed to hash password");

    // TODO: return new user
    let user_id = db
        .run(|con| {
            insert_into(users::table)
                .values(new_user)
                .returning(users::id)
                .get_result(con)
        })
        .await
        .expect("Failed to create user");

    Json(Protocol::LoginResponse(LoginResponse {
        key: session_token.to_string(),
        user: UserData {
            id: user_id,
            username: creds.username.clone(),
            display_name: None,
            currency: 0,
            lobby: None,
        },
    }))
}

#[post("/users", data = "<creds>")]
pub async fn login(
    creds: Json<Credentials>,
    db: Database,
    games: &State<RunningGames>,
) -> Json<Protocol> {
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
            &SaltString::from_b64(user.salt.as_str()).expect("User salt currupted")
        )
        .expect("Failed to verify login"),
        user.password
    );

    let session_token = Uuid::new_v4();

    db.run(move |con| {
        update(users::table)
            .filter(users::id.eq(user.id))
            .set((
                users::session_token.eq(Some(session_token)),
                users::session_expires
                    .eq(chrono::Utc::now().naive_utc() + chrono::Duration::hours(1)),
            ))
            .execute(con)
    })
    .await
    .expect("Failed to update session token");

    let mut game = None;

    for (id, g) in games.games.lock().await.iter() {
        if g.lock().await.has_user(user.id) {
            debug!("User {:?} is in game {:?}", user.id, id);
            game = Some(Channel::Game(*id));
            break;
        }
    }

    let channels = vec![
        db.run(move |con| {
            lobby_users::table
                .filter(lobby_users::user_id.eq(user.id))
                .select(lobby_users::lobby_id)
                .first::<i32>(con)
                .map(Channel::Lobby)
                .ok()
        })
        .await,
        game,
    ];

    for channel in channels.into_iter().flatten() {
        ActivePolls::join_user(channel, user.id);
    }

    Json(Protocol::LoginResponse(LoginResponse {
        key: session_token.to_string(),
        user: UserData {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            currency: user.currency,
            lobby: None,
        },
    }))
}

#[get("/users/@me")]
pub fn me(user: &User, lobby: Option<LobbyWithUsers>) -> Json<Protocol> {
    Json(Protocol::UserResponse(UserData {
        id: user.id,
        username: user.username.to_string(),
        display_name: user.display_name.clone(),
        currency: user.currency,
        lobby: lobby.map(|l| l.into()),
    }))
}

#[put("/users/display_name", data = "<display_name>")]
pub async fn set_display_name(
    db: Database,
    user: &User,
    display_name: Json<String>,
) -> Json<Protocol> {
    let user_id = user.id;
    if let Ok(display_name) = db
        .run(move |con| {
            diesel::update(users::table.filter(users::id.eq(user_id)))
                .set(users::display_name.eq(&display_name.0))
                .execute(con)?;

            QueryResult::Ok(display_name)
        })
        .await
    {
        Json(Protocol::DisplaynameResponse(display_name.0))
    } else {
        Json(Error::new_protocol(
            Status::InternalServerError.code,
            "Failed to update display name".to_string(),
        ))
    }
}
