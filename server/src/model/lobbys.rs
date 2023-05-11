use super::lobby_users::LobbyUser;
use super::lobby_users::NewLobbyUser;
use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;
use crate::model::users::User;
use crate::schema::{lobby_users, lobbys};
use crate::Database;

use crate::diesel::BelongingToDsl;
use chrono::NaiveDateTime;
use diesel::delete;
use diesel::dsl::count;
use diesel::dsl::not;
use diesel::{insert_into, QueryDsl};
use protocol::protocol::LobbyInfo;
use protocol::protocol::LobbyJoinRequest;
use protocol::protocol::Protocol;
use rocket::http::Status;
use rocket::request;
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::serde::json::Json;
use rocket::Request;

#[derive(Identifiable, Queryable, Clone, Debug)]
pub struct Lobby {
    pub id: i32,
    pub name: String,
    pub passphrase: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct LobbyWithUsers {
    pub lobby: Lobby,
    pub users: Vec<LobbyUser>,
}

#[derive(Insertable)]
#[table_name = "lobbys"]
pub struct NewLobby {
    pub name: String,
    pub passphrase: String,
}

impl Into<NewLobby> for LobbyJoinRequest {
    fn into(self) -> NewLobby {
        NewLobby {
            name: self.name,
            passphrase: self.passphrase,
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
                        if let Ok(lobby) = lobbys::table
                            .filter(
                                lobbys::id.eq_any(
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

#[get("/lobbys")]
pub async fn get_current_loby_info(lobby: LobbyWithUsers) -> Json<Protocol> {
    Json(Protocol::LOBBY_STATUS_RESPONSE(LobbyInfo {
        name: lobby.lobby.name,
        users: lobby.users.into_iter().map(|u| u.into()).collect(),
    }))
}

#[put("/lobbys", data = "<lobby>")]
pub async fn join_lobby(
    lobby: Json<LobbyJoinRequest>,
    user: &User,
    db: Database,
) -> (Status, Option<&'static str>) {
    let lobby = lobby.into_inner();
    let user = user.clone();
    if let Ok(_) = db
        .run(move |con| {
            let lobby = if let Ok(existing_lobby) = lobbys::table
                .filter(lobbys::name.eq(&lobby.name))
                .filter(lobbys::passphrase.eq(&lobby.passphrase))
                .first::<Lobby>(con)
            {
                match LobbyUser::belonging_to(&existing_lobby)
                    .select(count(lobby_users::id))
                    .first::<i64>(con)
                {
                    Ok(player_count) if player_count >= 8 => Err(LobbyError::Full),
                    _ => Ok(existing_lobby),
                }
            } else {
                if let Ok(results) = insert_into(lobbys::dsl::lobbys)
                    .values::<NewLobby>(lobby.into())
                    .get_results::<Lobby>(con)
                {
                    results.first().cloned().ok_or(LobbyError::Internal)
                } else {
                    Err(LobbyError::Internal)
                }
            };

            if let Ok(lobby) = lobby {
                debug!("Target lobby {:?} for user {:?}", lobby, user);
                if let Err(err) = delete(lobby_users::table)
                    .filter(lobby_users::user_id.eq(user.id))
                    .execute(con)
                {
                    error!("Failed to delete lobby user entries {:?}", err);
                    return Err(LobbyError::Internal);
                }
                if let Err(err) = insert_into(lobby_users::table)
                    .values::<NewLobbyUser>(NewLobbyUser::from_parents(&lobby, &user))
                    .execute(con)
                {
                    error!("Failed to create lobby user entry {:?}", err);
                    return Err(LobbyError::Internal);
                }
                // TODO: Refactor into cron
                if let Err(err) = delete(lobbys::table)
                    .filter(not(lobbys::id.eq_any(
                        lobby_users::table.select(lobby_users::lobby_id).distinct(),
                    )))
                    .execute(con)
                {
                    error!("Failed to delete empty lobbies {}", err)
                }
                Ok(())
            } else {
                Err(LobbyError::Internal)
            }
        })
        .await
    {
        return (Status::NoContent, Some(""));
    } else {
        return (Status::InternalServerError, Some("Failed to join lobby"));
    }
}
