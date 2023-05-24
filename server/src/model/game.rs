use crate::{
    schema::{game_users, games},
    Database,
};

use chrono::NaiveDateTime;
use diesel::{prelude::*, QueryDsl};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};

use super::users::User;

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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Game {
    type Error = GameError;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Outcome::Success(user) = req.guard::<&User>().await {
            let user = user.clone();
            if let Some(db) = req.guard::<Database>().await.succeeded() {
                return db
                    .run(move |con| {
                        if let Ok(game) = games::table
                            .filter(
                                games::id.eq_any(
                                    game_users::table
                                        .filter(
                                            game_users::user_id
                                                .eq(user.id)
                                                .and(game_users::placement.is_null()),
                                        )
                                        .select(game_users::game_id),
                                ),
                            )
                            .first::<Game>(con)
                        {
                            return Outcome::Success(game);
                        } else {
                            return Outcome::Forward(());
                        };
                    })
                    .await;
            }
            return Outcome::Failure((Status::ServiceUnavailable, GameError::Internal));
        }
        Outcome::Failure((Status::Unauthorized, GameError::Internal))
    }
}
