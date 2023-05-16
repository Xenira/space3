use super::lobby_users::LobbyUser;
use crate::diesel::{BelongingToDsl, ExpressionMethods, RunQueryDsl};
use crate::model::users::User;
use crate::schema::{lobbies, lobby_users};
use crate::service::lobby_service;
use crate::Database;

use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{prelude::*, QueryDsl};
use protocol::protocol::{Error, LobbyInfo, LobbyJoinRequest, Protocol};
use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    serde::json::Json,
    Request,
};

#[derive(Identifiable, Associations, Queryable, Clone, Debug)]
#[diesel(table_name = lobbies)]
#[diesel(belongs_to(User, foreign_key = master_id))]
pub struct Lobby {
    pub id: i32,
    pub name: String,
    pub passphrase: Option<String>,
    pub master_id: i32,
    pub start_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Lobby {
    pub fn into_lobby_info(&self, users: &Vec<LobbyUser>) -> LobbyInfo {
        LobbyInfo {
            name: self.name.clone(),
            master: users
                .iter()
                .find(|user| user.user_id == self.master_id)
                .map(|user| user.id)
                .unwrap_or_default(),
            users: users.iter().map(|user| user.clone().into()).collect(),
            start_at: self.start_at.map(|start| DateTime::from_utc(start, Utc)),
        }
    }
}

pub struct LobbyWithUsers {
    pub lobby: Lobby,
    pub users: Vec<LobbyUser>,
}

impl Into<LobbyInfo> for LobbyWithUsers {
    fn into(self) -> LobbyInfo {
        self.lobby.into_lobby_info(&self.users)
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = lobbies)]
pub struct NewLobby {
    pub name: String,
    pub passphrase: String,
    pub master_id: i32,
}

impl NewLobby {
    pub fn from_join_request(join_request: &LobbyJoinRequest, master_id: i32) -> Self {
        Self {
            name: join_request.name.clone(),
            passphrase: join_request.passphrase.clone(),
            master_id,
        }
    }
}

#[derive(Debug)]
pub enum LobbyError {
    Full,
    Internal,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LobbyWithUsers {
    type Error = LobbyError;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Outcome::Success(user) = req.guard::<&User>().await {
            let user = user.clone();
            if let Some(db) = req.guard::<Database>().await.succeeded() {
                return db
                    .run(move |con| {
                        if let Ok(lobby) = lobbies::table
                            .filter(
                                lobbies::id.eq_any(
                                    lobby_users::table
                                        .filter(lobby_users::user_id.eq(user.id))
                                        .select(lobby_users::lobby_id),
                                ),
                            )
                            .first::<Lobby>(con)
                        {
                            let users = LobbyUser::belonging_to(&lobby)
                                .load(con)
                                .unwrap_or_default();
                            return Outcome::Success(LobbyWithUsers { lobby, users });
                        } else {
                            return Outcome::Forward(());
                        };
                    })
                    .await;
            }
            return Outcome::Failure((Status::ServiceUnavailable, LobbyError::Internal));
        }
        Outcome::Failure((Status::Unauthorized, LobbyError::Internal))
    }
}

#[get("/lobbies")]
pub async fn get_current_loby_info(lobby: LobbyWithUsers) -> Json<Protocol> {
    Json(Protocol::LobbyStatusResponse(lobby.into()))
}

#[patch("/lobbies/ready")]
pub async fn toggle_ready_state(user: &User, lobby: LobbyWithUsers, db: Database) -> Status {
    for u in lobby.users {
        if u.user_id == user.id {
            let rdy = !u.ready;
            lobby_service::set_ready_state(&db, &u, rdy).await;
        }
    }

    Status::Ok
}

#[patch("/lobbies/start")]
pub async fn start_lobby_timer(user: &User, lobby: LobbyWithUsers, db: Database) -> Status {
    if user.id != lobby.lobby.master_id {
        return Status::Unauthorized;
    }

    lobby_service::start_lobby_timer(&db, &lobby).await;

    Status::Ok
}

#[patch("/lobbies/stop")]
pub async fn stop_lobby_timer(user: &User, lobby: LobbyWithUsers, db: Database) -> Status {
    if user.id != lobby.lobby.master_id {
        return Status::Unauthorized;
    }

    lobby_service::stop_lobby_timer(&db, lobby.lobby.id).await;

    Status::Ok
}

#[put("/lobbies", data = "<lobby>")]
pub async fn join_lobby(
    lobby: Json<LobbyJoinRequest>,
    user: &User,
    db: Database,
) -> (Status, Option<Json<Protocol>>) {
    match lobby_service::join_lobby(&db, lobby.into_inner(), user).await {
        Ok(lobby) => (Status::Ok, None),
        Err(LobbyError::Full) => (
            Status::Conflict,
            Some(Json(Error::new_protocol(
                Status::Conflict.code,
                "Lobby is full".to_string(),
            ))),
        ),
        Err(_) => (
            Status::InternalServerError,
            Some(Json(Error::new_protocol(
                Status::InternalServerError.code,
                "Failed to join lobby".to_string(),
            ))),
        ),
    }
}

#[delete("/lobbies")]
pub async fn leave_lobby(user: &User, db: Database) -> Json<Protocol> {
    lobby_service::remove_user_from_lobbies(&db, user).await;

    Json(Protocol::LobbyLeaveResponse)
}
