use super::game::GameGuard;
use crate::model::game_users::GameUser;
use crate::model::users::User;
use crate::schema::game_user_characters;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use protocol::protocol::{Error, GameUserInfo, Protocol};
use rocket::http::Status;
use rocket::serde::json::Json;

#[derive(Identifiable, Associations, Queryable, Clone, Debug)]
#[diesel(belongs_to(GameUser))]
pub struct GameUserCharacter {
    pub id: i32,
    pub game_user_id: i32,
    pub character_id: i32,
    pub position: i32,
    pub upgraded: bool,
    pub attack_bonus: i32,
    pub defense_bonus: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl GameUserCharacter {
    pub fn new(
        game_user_id: i32,
        character_id: i32,
        position: i32,
        upgraded: bool,
        attack_bonus: i32,
        defense_bonus: i32,
    ) -> Self {
        Self {
            id: 0,
            game_user_id,
            character_id,
            position,
            upgraded,
            attack_bonus,
            defense_bonus,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = game_user_characters)]
pub struct NewGameUserCharacter {
    pub game_user_id: i32,
    pub character_id: i32,
    pub position: i32,
    pub upgraded: bool,
    pub attack_bonus: i32,
    pub defense_bonus: i32,
}

#[derive(AsChangeset)]
#[diesel(table_name = game_user_characters)]
pub struct GameUserCharacterUpdate {
    pub position: Option<i32>,
    pub upgraded: Option<bool>,
    pub attack_bonus: Option<i32>,
    pub defense_bonus: Option<i32>,
}

impl GameUserCharacterUpdate {
    pub fn new() -> Self {
        Self {
            position: None,
            upgraded: None,
            attack_bonus: None,
            defense_bonus: None,
        }
    }

    pub fn with_position(mut self, position: i32) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_upgraded(mut self, upgraded: bool) -> Self {
        self.upgraded = Some(upgraded);
        self
    }

    pub fn with_attack_bonus(mut self, attack_bonus: i32) -> Self {
        self.attack_bonus = Some(attack_bonus);
        self
    }

    pub fn with_defense_bonus(mut self, defense_bonus: i32) -> Self {
        self.defense_bonus = Some(defense_bonus);
        self
    }
}

#[derive(Debug)]
pub enum GameUserCharacterError {
    Internal,
}

#[get("/games/characters")]
pub async fn get_board(game: GameGuard, user: &User) -> Json<Protocol> {
    let game = game.0.lock().await;
    if let Some(game_user) = game.get_user(user.id) {
        Json(Protocol::BoardResponse(game_user.board.to_vec()))
    } else {
        Json(Error::new_protocol_response(
            Status::NotFound.code,
            "Character not found".to_string(),
            Protocol::CharacterMoveRequest,
        ))
    }
}

#[put("/games/characters/<character_idx>/<target_idx>")]
pub async fn move_character(
    game: GameGuard,
    user: &User,
    character_idx: usize,
    target_idx: usize,
) -> Json<Protocol> {
    let mut game = game.0.lock().await;
    if let Some(game_user) = game.get_user_mut(user.id) {
        if game_user.move_character(character_idx, target_idx).is_ok() {
            Json(Protocol::BoardResponse(game_user.board.to_vec()))
        } else {
            Json(Error::new_protocol_response(
                Status::NotFound.code,
                "Character not found".to_string(),
                Protocol::CharacterMoveRequest,
            ))
        }
    } else {
        Json(Error::new_protocol_response(
            Status::NotFound.code,
            "Character not found".to_string(),
            Protocol::CharacterMoveRequest,
        ))
    }
}

#[delete("/games/characters/<character_idx>")]
pub async fn sell_character(user: &User, game: GameGuard, character_idx: usize) -> Json<Protocol> {
    let mut game = game.0.lock().await;
    let Some(game_user) = game.get_user_mut(user.id) else {
        return Json(Error::new_protocol_response(
            Status::NotFound.code,
            "Character not found".to_string(),
            Protocol::CharacterMoveRequest,
        ))
    };

    if game_user.sell(character_idx).is_ok() {
        Json(Protocol::SellResponse(
            GameUserInfo {
                experience: game_user.experience,
                health: game_user.health,
                money: game_user.money,
                name: user.username.clone(),
                avatar: game_user.god.clone().and_then(|g| Some(g.id)),
            },
            game_user.board.to_vec(),
        ))
    } else {
        Json(Error::new_protocol_response(
            Status::NotFound.code,
            "Character not found".to_string(),
            Protocol::CharacterMoveRequest,
        ))
    }
}
