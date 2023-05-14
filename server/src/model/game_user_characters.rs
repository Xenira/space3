use crate::model::game_users::GameUser;
use crate::schema::game_user_characters;
use crate::Database;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome};
use rocket::Request;

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

#[derive(Insertable)]
#[table_name = "game_user_characters"]
pub struct NewGameUserCharacter {
    pub game_user_id: i32,
    pub character_id: i32,
    pub position: i32,
    pub upgraded: bool,
    pub attack_bonus: i32,
    pub defense_bonus: i32,
}

#[derive(AsChangeset)]
#[table_name = "game_user_characters"]
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

#[derive(Debug)]
pub struct GameUserCharacters(pub Vec<Option<GameUserCharacter>>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GameUserCharacters {
    type Error = GameUserCharacterError;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Outcome::Success(game_user) = req.guard::<GameUser>().await {
            let game_user = game_user.clone();
            if let Some(db) = req.guard::<Database>().await.succeeded() {
                return db
                    .run(move |con| {
                        if let Ok(shop) = GameUserCharacter::belonging_to(&game_user)
                            .load::<GameUserCharacter>(con)
                        {
                            let character_idx = 0..12;
                            Outcome::Success(GameUserCharacters(
                                character_idx
                                    .map(|i| {
                                        shop.iter().find(|c| c.position == i).map(|c| c.clone())
                                    })
                                    .collect::<Vec<_>>(),
                            ))
                        } else {
                            return Outcome::Forward(());
                        }
                    })
                    .await;
            }
            return Outcome::Failure((Status::ServiceUnavailable, Self::Error::Internal));
        }
        Outcome::Failure((Status::Unauthorized, Self::Error::Internal))
    }
}
