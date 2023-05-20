use crate::model::game_users::GameUser;
use crate::model::users::User;
use crate::schema::{game_user_characters, game_users};
use crate::service::character_service;
use crate::Database;
use chrono::NaiveDateTime;
use diesel::{delete, prelude::*, update};
use protocol::protocol::{Error, GameUserInfo, Protocol};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome};
use rocket::serde::json::Json;
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

#[get("/games/characters")]
pub async fn get_board(db: Database, game_user: GameUser) -> Json<Protocol> {
    if let Ok(board) = character_service::get_board(&db, game_user.id).await {
        Json(Protocol::BoardResponse(board))
    } else {
        Json(Error::new_protocol_response(
            Status::InternalServerError.code,
            "Could not get board".to_string(),
            Protocol::CharacterMoveRequest,
        ))
    }
}

#[put("/games/characters/<character_idx>/<target_idx>")]
pub async fn move_character(
    db: Database,
    game_user_characters: GameUserCharacters,
    character_idx: u8,
    target_idx: u8,
) -> Json<Protocol> {
    if character_idx == target_idx {
        return Json(Error::new_protocol_response(
            Status::BadRequest.code,
            "Cannot move character to same position".to_string(),
            Protocol::CharacterMoveRequest,
        ));
    }

    if let Some(source_character) = game_user_characters.0[character_idx as usize].clone() {
        let source_character_id = source_character.id;
        if let Some(target_character) = game_user_characters.0[target_idx as usize].clone() {
            let target_character_id = target_character.id;
            db.run(move |con| {
                con.build_transaction().deferrable().run(|con| {
                    update(game_user_characters::table)
                        .filter(game_user_characters::id.eq(source_character_id))
                        .set(game_user_characters::position.eq(target_idx as i32))
                        .execute(con)
                        .unwrap();
                    update(game_user_characters::table)
                        .filter(game_user_characters::id.eq(target_character_id))
                        .set(game_user_characters::position.eq(character_idx as i32))
                        .execute(con)
                        .unwrap();
                    QueryResult::Ok(())
                })
            })
            .await;
        } else {
            db.run(move |con| {
                update(game_user_characters::table)
                    .filter(game_user_characters::id.eq(source_character.id))
                    .set(game_user_characters::position.eq(target_idx as i32))
                    .execute(con)
                    .unwrap();
            })
            .await;
        }

        if let Ok(board) = character_service::get_board(&db, source_character.game_user_id).await {
            Json(Protocol::BoardResponse(board))
        } else {
            Json(Error::new_protocol_response(
                Status::InternalServerError.code,
                "Could not get board".to_string(),
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
pub async fn sell_character(
    db: Database,
    user: &User,
    game_user: GameUser,
    game_user_characters: GameUserCharacters,
    character_idx: u8,
) -> Json<Protocol> {
    if let Some(character) = game_user_characters.0[character_idx as usize].clone() {
        db.run(move |con| {
            con.transaction(|con| {
                delete(game_user_characters::table)
                    .filter(game_user_characters::id.eq(character.id))
                    .execute(con)?;

                update(game_users::table)
                    .filter(game_users::id.eq(character.game_user_id))
                    .set(game_users::credits.eq(game_users::credits + 1))
                    .execute(con)?;

                QueryResult::Ok(())
            })
        })
        .await;

        if let Ok(board) = character_service::get_board(&db, character.game_user_id).await {
            Json(Protocol::SellResponse(
                GameUserInfo {
                    experience: game_user.experience,
                    health: game_user.health,
                    money: game_user.credits + 1,
                    name: user.username.clone(),
                    avatar: game_user.avatar_id,
                },
                board,
            ))
        } else {
            Json(Error::new_protocol_response(
                Status::InternalServerError.code,
                "Could not get board".to_string(),
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
