use crate::model::{game::Game, users::User};
use crate::schema::game_users;
use crate::service::combat_service;
use crate::Database;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use protocol::protocol::{GameOpponentInfo, GameUserInfo, Protocol};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::Request;

#[derive(Identifiable, Associations, Queryable, Clone, Default, PartialEq, Debug)]
#[diesel(belongs_to(Game))]
#[diesel(belongs_to(User))]
pub struct GameUser {
    pub id: i32,
    pub game_id: i32,
    pub user_id: i32,
    pub display_name: String,
    pub avatar_id: Option<i32>,
    pub experience: i32,
    pub health: i32,
    pub credits: i32,
    pub placement: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl GameUser {
    pub fn opponent_info(&self, is_next_opponent: bool) -> GameOpponentInfo {
        GameOpponentInfo {
            name: self.display_name.clone(),
            experience: self.experience,
            health: self.health,
            character_id: self.avatar_id.unwrap_or(0),
            is_next_opponent,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = game_users)]
pub struct NewGameUser {
    pub game_id: i32,
    pub user_id: i32,
    pub display_name: String,
    pub experience: i32,
    pub health: i32,
    pub credits: i32,
}

impl NewGameUser {
    pub fn from_parents(game_id: i32, user_id: i32, display_name: impl Into<String>) -> Self {
        Self {
            game_id,
            user_id,
            display_name: display_name.into(),
            experience: 0,
            health: 10,
            credits: 0,
        }
    }
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = game_users)]
pub struct GameUserUpdate {
    pub experience: Option<i32>,
    pub health: Option<i32>,
    pub credits: Option<i32>,
}

#[derive(Debug)]
pub struct GameUsers(pub Vec<GameUser>);

#[derive(Debug)]
pub enum GameUserError {
    Internal,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GameUser {
    type Error = GameUserError;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Outcome::Success(user) = req.guard::<&User>().await {
            let user = user.clone();
            if let Some(db) = req.guard::<Database>().await.succeeded() {
                return db
                    .run(move |con| {
                        if let Ok(user) = game_users::table
                            .filter(
                                game_users::user_id
                                    .eq(user.id)
                                    .and(game_users::placement.is_null()),
                            )
                            .first::<GameUser>(con)
                        {
                            return Outcome::Success(user);
                        } else {
                            return Outcome::Forward(());
                        };
                    })
                    .await;
            }
            return Outcome::Failure((Status::ServiceUnavailable, Self::Error::Internal));
        }
        Outcome::Failure((Status::Unauthorized, Self::Error::Internal))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GameUsers {
    type Error = GameUserError;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Outcome::Success(game) = req.guard::<Game>().await {
            if let Some(db) = req.guard::<Database>().await.succeeded() {
                return db
                    .run(move |con| {
                        if let Ok(users) = game_users::table
                            .filter(game_users::game_id.eq(game.id))
                            .load::<GameUser>(con)
                        {
                            return Outcome::Success(GameUsers(users));
                        } else {
                            return Outcome::Forward(());
                        };
                    })
                    .await;
            }
            return Outcome::Failure((Status::ServiceUnavailable, Self::Error::Internal));
        }
        Outcome::Failure((Status::Unauthorized, Self::Error::Internal))
    }
}

#[get("/games/users/me")]
pub async fn get_own_user(user: &User, game_user: GameUser) -> Json<Protocol> {
    Json(Protocol::GameUserInfoResponse(GameUserInfo {
        experience: game_user.experience,
        health: game_user.health,
        money: game_user.credits,
        name: user.username.clone(),
        avatar: game_user.avatar_id,
    }))
}

#[get("/games/users")]
pub async fn get_users(game: Game, game_user: GameUser, game_users: GameUsers) -> Json<Protocol> {
    let pairings = combat_service::get_pairing(game.current_round + 1, &game_users.0);
    let next_opponent = pairings
        .iter()
        .find(|p| p.0.id == game_user.id || p.1.id == game_user.id)
        .map(|p| {
            if p.0.id == game_user.id {
                p.1.id
            } else {
                p.0.id
            }
        })
        .unwrap_or(0);

    debug!(
        "Next opponent: {} based on pairings: {:?}",
        next_opponent, pairings
    );

    Json(Protocol::GameUsersResponse(
        game_users
            .0
            .iter()
            .map(|u| u.opponent_info(u.id == next_opponent))
            .collect::<Vec<_>>(),
    ))
}
