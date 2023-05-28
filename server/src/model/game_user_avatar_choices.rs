use chrono::NaiveDateTime;
use diesel::prelude::*;
use protocol::protocol::Protocol;

use crate::{
    model::{game_users::GameUser, users::User},
    schema::game_user_avatar_choices,
    service::game_service::notify_users,
};
use protocol::gods::GODS;
use rocket::{http::Status, serde::json::Json};

use super::game::{Game, GameGuard};

#[derive(Identifiable, Queryable, Associations, Clone, Debug)]
#[diesel(belongs_to(Game))]
#[diesel(belongs_to(GameUser))]
pub struct GameUserAvatarChoice {
    pub id: i32,
    pub game_id: i32,
    pub game_user_id: i32,
    pub avatar_id: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = game_user_avatar_choices)]
pub struct NewGameUserAvatarChoice {
    pub game_id: i32,
    pub game_user_id: i32,
    pub avatar_id: i32,
}

impl NewGameUserAvatarChoice {
    pub fn from_parents(game_id: i32, game_user_id: i32, avatar_id: i32) -> Self {
        Self {
            game_id,
            game_user_id,
            avatar_id,
        }
    }
}

pub enum GameUserAvatarChoiceError {
    AvatarNotAvailable,
    AvatarAlreadyChosen,
}

#[put("/games/avatar/<avatar_id>")]
pub async fn select_avatar(game: GameGuard, user: &User, avatar_id: i32) -> Json<Protocol> {
    let mut game = game.0.lock().await;
    if let Some(game_user) = game.get_user_mut(user.id) {
        if game_user.god.is_some() {
            return Json(protocol::protocol::Error::new_protocol(
                Status::BadRequest.code,
                "Avatar already chosen".to_string(),
            ));
        }

        if game_user.god_choices.contains(&avatar_id) {
            let god = GODS[avatar_id as usize].clone();
            game_user.god = Some(god.clone());

            if !game.players.iter().any(|p| p.god.is_none()) {
                game.next_turn().await;
            }

            notify_users(&game).await;
            Json(Protocol::AvatarSelectResponse(god))
        } else {
            Json(protocol::protocol::Error::new_protocol(
                Status::Conflict.code,
                "Avatar not available".to_string(),
            ))
        }
    } else {
        Json(protocol::protocol::Error::new_protocol(
            Status::BadRequest.code,
            "User not in game".to_string(),
        ))
    }
}
