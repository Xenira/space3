use super::game::GameGuard;
use crate::{
    model::{game::Game, users::User},
    schema::game_users,
    service::combat_service,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use protocol::protocol::{GameUserInfo, Protocol};
use rocket::serde::json::Json;

#[derive(Identifiable, Associations, Queryable, Clone, Default, PartialEq, Debug)]
#[diesel(belongs_to(Game))]
#[diesel(belongs_to(User))]
pub struct GameUser {
    pub id: i32,
    pub game_id: i32,
    pub user_id: Option<i32>,
    pub display_name: String,
    pub avatar_id: Option<i32>,
    pub experience: i32,
    pub health: i32,
    pub credits: i32,
    pub placement: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl GameUser {}

#[derive(Insertable)]
#[diesel(table_name = game_users)]
pub struct NewGameUser {
    pub game_id: i32,
    pub user_id: Option<i32>,
    pub display_name: String,
    pub avatar_id: Option<i32>,
    pub experience: i32,
    pub health: i32,
    pub credits: i32,
}

impl NewGameUser {
    pub fn from_parents(
        game_id: i32,
        user_id: Option<i32>,
        display_name: impl Into<String>,
        god_id: Option<i32>,
    ) -> Self {
        Self {
            game_id,
            user_id,
            display_name: display_name.into(),
            avatar_id: god_id,
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

#[get("/games/users/me")]
pub async fn get_own_user(user: &User, game: GameGuard) -> Json<Protocol> {
    let mut game = game.0.lock().await;
    let game_user = game
        .players
        .iter_mut()
        .find(|p| p.user_id == Some(user.id))
        .ok_or(GameUserError::Internal)
        .unwrap();

    Json(Protocol::GameUserInfoResponse(GameUserInfo {
        experience: game_user.experience,
        health: game_user.health,
        money: game_user.money,
        name: user.username.clone(),
        avatar: game_user.god.clone().map(|g| g.id),
    }))
}

#[get("/games/users")]
pub async fn get_users(game: GameGuard, user: &User) -> Json<Protocol> {
    let game = game.0.lock().await;
    let id = game.get_user(user.id).unwrap().id;

    let pairings =
        combat_service::get_pairing(game.turn.into(), game.players.iter().collect::<Vec<_>>());

    let next_opponent = pairings
        .iter()
        .find(|p| p.0 == id || p.1 == id)
        .map(|p| if p.0 == id { p.1 } else { p.0 })
        .unwrap();

    debug!(
        "Next opponent: {} based on pairings: {:?}",
        next_opponent, pairings
    );

    Json(Protocol::GameUsersResponse(
        game.players
            .iter()
            .map(|u| u.opponent_info(u.id == next_opponent))
            .collect::<Vec<_>>(),
    ))
}
