use crate::{
    model::{game_user_characters::GameUserCharacters, game_users::GameUser, users::User},
    schema::{game_user_characters, game_users, shops},
    service::character_service,
    Database,
};
use chrono::NaiveDateTime;
use diesel::{delete, insert_into, prelude::*, update};
use protocol::{
    characters::CHARACTERS,
    protocol::{BuyRequest, CharacterInstance, Error, GameUserInfo, Protocol},
};
use rand::seq::SliceRandom;
use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    serde::json::Json,
    Request,
};
use serde::{Deserialize, Serialize};

use super::game_user_characters::{GameUserCharacter, NewGameUserCharacter};

#[derive(Queryable, Identifiable, Associations, Serialize, Deserialize, Clone, Debug)]
#[diesel(belongs_to(GameUser))]
pub struct Shop {
    pub id: i32,
    pub game_user_id: i32,
    pub character_ids: Vec<Option<i32>>,
    pub locked: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = shops)]
pub struct NewShop {
    game_user_id: i32,
    character_ids: Vec<Option<i32>>,
}

impl NewShop {
    pub fn new(game_user_id: i32) -> Self {
        Self {
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Shop {
    type Error = ShopError;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Outcome::Success(game_user) = req.guard::<GameUser>().await {
            let game_user = game_user.clone();
            if let Some(db) = req.guard::<Database>().await.succeeded() {
                return db
                    .run(move |con| {
                        if let Ok(shop) = Shop::belonging_to(&game_user).first::<Shop>(con) {
                            return Outcome::Success(shop);
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

#[get("/games/shop")]
pub async fn get_shop(db: crate::Database, game_user: GameUser) -> Json<Protocol> {
    let game_user = game_user.clone();
    let shop = db
        .run(move |c| {
            if let Ok(shop) = shops::table
                .filter(shops::game_user_id.eq(game_user.id))
                .first::<Shop>(c)
            {
                shop
            } else {
                let mut new_shop = NewShop::new(game_user.id);
                let characters = CHARACTERS.clone();
                while new_shop.character_ids.len() < 5 {
                    new_shop
                        .character_ids
                        .push(Some(characters.choose(&mut rand::thread_rng()).unwrap().id));
                }

                let new_shop = diesel::insert_into(shops::table)
                    .values(&new_shop)
                    .returning(shops::table::all_columns())
                    .get_result::<Shop>(c)
                    .unwrap();

                new_shop
            }
        })
        .await;

    Json(Protocol::GameShopResponse(
        shop.character_ids
            .into_iter()
            .map(|c| c.and_then(|c| Some(CharacterInstance::from(&CHARACTERS[c as usize]))))
            .collect::<Vec<_>>(),
    ))
}

#[post("/games/shop")]
pub async fn reroll_shop(db: Database, game_user: GameUser) -> Json<Protocol> {
    if game_user.credits <= 0 {
        return Json(Error::new_protocol_response(
            Status::PaymentRequired.code,
            "Not enough credits".to_string(),
            Protocol::RerollShopRequest,
        ));
    }

    let game_user_id = game_user.id;
    if let Ok(shop) = db
        .run(move |c| {
            c.transaction(|c| {
                delete(shops::table.filter(shops::game_user_id.eq(game_user_id))).execute(c)?;

                let mut new_shop = NewShop::new(game_user.id);
                let characters = CHARACTERS.clone();
                while new_shop.character_ids.len() < 5 {
                    new_shop
                        .character_ids
                        .push(Some(characters.choose(&mut rand::thread_rng()).unwrap().id));
                }

                diesel::insert_into(shops::table)
                    .values(&new_shop)
                    .execute(c)
                    .unwrap();

                update(game_users::table)
                    .filter(game_users::id.eq(game_user_id))
                    .set(game_users::credits.eq(game_users::credits - 1))
                    .execute(c)?;

                QueryResult::Ok(new_shop)
            })
        })
        .await
    {
        Json(Protocol::GameShopResponse(
            shop.character_ids
                .into_iter()
                .map(|c| {
                    c.and_then(|c| Some(CharacterInstance::from(&CHARACTERS[c as usize].clone())))
                })
                .collect::<Vec<_>>(),
        ))
    } else {
        Json(Error::new_protocol_response(
            Status::InternalServerError.code,
            "Internal server error".to_string(),
            Protocol::RerollShopRequest,
        ))
    }
}

#[post("/games/shop/buy", data = "<buy_request>")]
pub async fn buy_character(
    db: Database,
    user: &User,
    game_user: GameUser,
    shop: Shop,
    mut game_user_characters: GameUserCharacters,
    buy_request: Json<BuyRequest>,
) -> Json<Protocol> {
    let character =
        if let Some(Some(character)) = shop.character_ids.get(buy_request.character_idx as usize) {
            let character = CHARACTERS[*character as usize].clone();
            if character.cost > game_user.credits {
                return Json(Error::new_protocol_response(
                    Status::PaymentRequired.code,
                    "Not enough credits".to_string(),
                    Protocol::BuyRequest(buy_request.into_inner()),
                ));
            }
            character
        } else {
            return Json(Error::new_protocol_response(
                Status::UnprocessableEntity.code,
                "Invalid character index".to_string(),
                Protocol::BuyRequest(buy_request.into_inner()),
            ));
        };

    if let Some(Some(_)) = game_user_characters
        .0
        .get(buy_request.character_idx as usize)
    {
        // TODO: Tyr to move current character to free slot
        return Json(Error::new_protocol_response(
            Status::UnprocessableEntity.code,
            "Character slot alredy occupied".to_string(),
            Protocol::BuyRequest(buy_request.into_inner()),
        ));
    }

    game_user_characters.0.insert(
        buy_request.target_idx.into(),
        Some(GameUserCharacter::new(
            game_user.id,
            character.id,
            buy_request.target_idx.into(),
            false,
            0,
            0,
        )),
    );

    let game_user_id = game_user.id;
    let character_id = character.id;
    let shop_id = shop.id;
    let mut shop_character_ids = shop.character_ids.clone();
    shop_character_ids[buy_request.character_idx as usize] = None;
    let request = buy_request.clone();
    if let Ok(shop_character_ids) = db
        .run(move |c| {
            c.build_transaction().run(move |c| {
                insert_into(game_user_characters::table)
                    .values(&NewGameUserCharacter {
                        game_user_id,
                        character_id,
                        position: buy_request.character_idx as i32,
                        upgraded: false,
                        attack_bonus: 0,
                        defense_bonus: 0,
                    })
                    .execute(c)?;

                update(shops::table)
                    .filter(shops::id.eq(shop_id))
                    .set(shops::character_ids.eq(shop_character_ids.clone()))
                    .execute(c)?;

                update(game_users::table)
                    .filter(game_users::id.eq(game_user_id))
                    .set(game_users::credits.eq(game_users::credits - character.cost))
                    .execute(c)?;

                QueryResult::Ok(shop_character_ids)
            })
        })
        .await
    {
        Json(Protocol::BuyResponse(
            GameUserInfo {
                experience: game_user.experience,
                health: game_user.health,
                money: game_user.credits - character.cost,
                name: user.username.clone(),
                avatar: game_user.avatar_id,
            },
            shop_character_ids
                .iter()
                .map(|c| {
                    c.and_then(|c| Some(CharacterInstance::from(&CHARACTERS[c as usize].clone())))
                })
                .collect::<Vec<_>>(),
            character_service::get_board(&db, game_user_id)
                .await
                .unwrap(),
        ))
    } else {
        Json(Error::new_protocol_response(
            Status::InternalServerError.code,
            "Internal server error".to_string(),
            Protocol::BuyRequest(request.into_inner()),
        ))
    }
}
