use crate::{
    model::{game::Game, game_users::GameUser, users::User},
    schema::shops,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use protocol::protocol::{BuyRequest, Error, GameUserInfo, Protocol};
use rocket::{http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};

use super::game::GameGuard;

#[derive(Queryable, Identifiable, Associations, Serialize, Deserialize, Clone, Debug)]
#[diesel(belongs_to(GameUser))]
#[diesel(belongs_to(Game))]
pub struct Shop {
    pub id: i32,
    pub game_id: i32,
    pub game_user_id: i32,
    pub character_ids: Vec<Option<i32>>,
    pub locked: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = shops)]
pub struct NewShop {
    game_id: i32,
    game_user_id: i32,
    character_ids: Vec<Option<i32>>,
}

impl From<&GameUser> for NewShop {
    fn from(game_user: &GameUser) -> Self {
        Self {
            game_id: game_user.game_id,
            game_user_id: game_user.id,
            character_ids: vec![],
        }
    }
}

impl NewShop {
    pub fn new(game_id: i32, game_user_id: i32) -> Self {
        Self {
            game_id,
            game_user_id,
            character_ids: vec![],
        }
    }
}

#[derive(AsChangeset, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = shops)]
pub struct ShopUpdate {
    character_ids: Vec<Option<i32>>,
}

impl ShopUpdate {
    pub fn new() -> Self {
        Self {
            character_ids: vec![None; 5],
        }
    }
}

#[derive(Debug)]
pub enum ShopError {
    Internal,
}

#[get("/games/shops")]
pub async fn get_shop(game: GameGuard, user: &User) -> Json<Protocol> {
    let game = game.0.lock().await;
    if let Some(game_user) = game.get_user(user.id) {
        Json(Protocol::GameShopResponse(
            GameUserInfo {
                experience: game_user.experience,
                health: game_user.health,
                money: game_user.money,
                name: game_user.display_name.to_string(),
                avatar: game_user.god.clone().and_then(|g| Some(g.id)),
            },
            game_user.shop.locked,
            game_user.shop.characters.clone(),
        ))
    } else {
        Json(Error::new_protocol(
            Status::InternalServerError.code,
            "Internal server error".to_string(),
        ))
    }
}

#[post("/games/shops")]
pub async fn reroll_shop(game: GameGuard, user: &User) -> Json<Protocol> {
    let mut game = game.0.lock().await;
    if let Some(game_user) = game.get_user_mut(user.id) {
        // TODO: Handle error
        if game_user.reroll().is_ok() {
            Json(Protocol::GameShopResponse(
                GameUserInfo {
                    experience: game_user.experience,
                    health: game_user.health,
                    money: game_user.money,
                    name: game_user.display_name.to_string(),
                    avatar: game_user.god.clone().and_then(|g| Some(g.id)),
                },
                false,
                game_user.shop.characters.clone(),
            ))
        } else {
            Json(Error::new_protocol_response(
                Status::InternalServerError.code,
                "Internal server error".to_string(),
                Protocol::RerollShopRequest,
            ))
        }
    } else {
        Json(Error::new_protocol_response(
            Status::InternalServerError.code,
            "Internal server error".to_string(),
            Protocol::RerollShopRequest,
        ))
    }
}

#[patch("/games/shops")]
pub async fn toggle_lock_shop(game: GameGuard, user: &User) -> Json<Protocol> {
    let mut game = game.0.lock().await;
    if let Some(user) = game.get_user_mut(user.id) {
        user.shop.locked = !user.shop.locked;

        Json(Protocol::GameShopResponse(
            GameUserInfo {
                experience: user.experience,
                health: user.health,
                money: user.money,
                name: user.display_name.to_string(),
                avatar: user.god.clone().and_then(|g| Some(g.id)),
            },
            user.shop.locked,
            user.shop.characters.clone(),
        ))
    } else {
        Json(Error::new_protocol(
            Status::InternalServerError.code,
            "Internal server error".to_string(),
        ))
    }
}

#[post("/games/shops/buy", data = "<buy_request>")]
pub async fn buy_character(
    user: &User,
    game: GameGuard,
    buy_request: Json<BuyRequest>,
) -> Json<Protocol> {
    let mut game = game.0.lock().await;
    if let Some(game_user) = game.get_user_mut(user.id) {
        if game_user
            .buy(
                buy_request.character_idx as usize,
                buy_request.target_idx as usize,
            )
            .is_ok()
        {
            Json(Protocol::BuyResponse(
                GameUserInfo {
                    experience: game_user.experience,
                    health: game_user.health,
                    money: game_user.money,
                    name: game_user.display_name.to_string(),
                    avatar: game_user.god.clone().and_then(|g| Some(g.id)),
                },
                game_user.shop.characters.clone(),
                game_user.board.to_vec(),
            ))
        } else {
            Json(Error::new_protocol_response(
                Status::InternalServerError.code,
                "Internal server error".to_string(),
                Protocol::BuyRequest(buy_request.into_inner()),
            ))
        }
    } else {
        Json(Error::new_protocol_response(
            Status::InternalServerError.code,
            "Internal server error".to_string(),
            Protocol::BuyRequest(buy_request.into_inner()),
        ))
    }
}
