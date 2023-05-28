use super::users::User;
use crate::{game::game_instance::GameInstance, schema::games, RunningGames};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    tokio::sync::Mutex,
    Request, State,
};
use std::sync::Arc;

#[derive(Identifiable, Queryable, Clone, Debug)]
pub struct Game {
    pub id: i32,
    pub next_battle: Option<NaiveDateTime>,
    pub current_round: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = games)]
pub struct NewGame {
    pub next_battle: Option<NaiveDateTime>,
    pub current_round: i32,
}

impl NewGame {
    pub fn new() -> Self {
        Self {
            next_battle: Some(chrono::Utc::now().naive_utc() + chrono::Duration::seconds(30)),
            current_round: 0,
        }
    }
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = games)]
pub struct GameUpdate {
    pub next_battle: Option<NaiveDateTime>,
    pub current_round: Option<i32>,
}

impl GameUpdate {
    pub fn new() -> Self {
        Self {
            next_battle: None,
            current_round: None,
        }
    }
}

#[derive(Debug)]
pub enum GameError {
    Internal,
}

pub struct GameGuard(pub Arc<Mutex<GameInstance>>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GameGuard {
    type Error = GameError;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Outcome::Success(user) = req.guard::<&User>().await {
            let user = user.clone();
            let games = req
                .guard::<&State<RunningGames>>()
                .await
                .unwrap()
                .games
                .clone()
                .lock_owned()
                .await;

            for game in games.values() {
                if game.lock().await.has_user(user.id) {
                    return Outcome::Success(GameGuard(game.clone()));
                }
            }

            return Outcome::Failure((Status::ServiceUnavailable, GameError::Internal));
        }
        Outcome::Failure((Status::Unauthorized, GameError::Internal))
    }
}
