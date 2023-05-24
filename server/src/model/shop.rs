use crate::{
    model::{
        game::Game, game_user_characters::GameUserCharacters, game_users::GameUser, users::User,
    },
    schema::{game_users, shops},
    service::{character_service, shop_service},
    Database,
};
use chrono::NaiveDateTime;
use diesel::{delete, prelude::*, update};
use protocol::{
    characters::CHARACTERS,
    protocol::{BuyRequest, Error, GameUserInfo, Protocol},
};
use rand::seq::SliceRandom;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    serde::json::Json,
    Request,
};
use serde::{Deserialize, Serialize};

use super::game_user_characters::GameUserCharacter;

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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Shop {
    type Error = ShopError;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Outcome::Success(game_user) = req.guard::<GameUser>().await {
            let game_user = game_user.clone();
            if let Some(db) = req.guard::<Database>().await.succeeded() {
                return db
                    .run(
                        move |con| match Shop::belonging_to(&game_user).first::<Shop>(con) {
                            Ok(shop) => Outcome::Success(shop),
                            Err(err) => {
                                warn!("Shop not found for game user {}: {}", game_user.id, err);
                                Outcome::Forward(())
                            }
                        },
                    )
                    .await;
            }
            return Outcome::Failure((Status::ServiceUnavailable, Self::Error::Internal));
        }
        Outcome::Failure((Status::Unauthorized, Self::Error::Internal))
    }
}

#[get("/games/shops")]
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
                let mut new_shop = NewShop::from(&game_user);
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
        shop.locked,
        shop_service::get_shop(&shop.character_ids),
    ))
}

#[post("/games/shops")]
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

                let mut new_shop = NewShop::from(&game_user);
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
            false,
            shop_service::get_shop(&shop.character_ids),
        ))
    } else {
        Json(Error::new_protocol_response(
            Status::InternalServerError.code,
            "Internal server error".to_string(),
            Protocol::RerollShopRequest,
        ))
    }
}

#[patch("/games/shops")]
pub async fn toggle_lock_shop(db: Database, shop: Shop) -> Json<Protocol> {
    db.run(move |c| {
        update(shops::table.filter(shops::id.eq(shop.id)))
            .set(shops::locked.eq(!shop.locked))
            .execute(c)
    })
    .await;

    Json(Protocol::GameShopResponse(
        !shop.locked,
        shop_service::get_shop(&shop.character_ids),
    ))
}

#[post("/games/shops/buy", data = "<buy_request>")]
pub async fn buy_character(
    db: Database,
    user: &User,
    game_user: GameUser,
    shop: Shop,
    game_user_characters: GameUserCharacters,
    buy_request: Json<BuyRequest>,
) -> Json<Protocol> {
    let character =
        if let Some(Some(character)) = shop.character_ids.get(buy_request.character_idx as usize) {
            let character = CHARACTERS[*character as usize].clone();
            if character.cost as i32 > game_user.credits {
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

    if game_user_characters
        .0
        .iter()
        .filter_map(|c| c.as_ref())
        .filter(|c| c.character_id == character.id && !c.upgraded)
        .count()
        == 2
    {
        // Upgrade character
        match shop_service::upgrade_character(
            &db,
            &game_user,
            &character,
            shop,
            buy_request.character_idx,
            buy_request.target_idx,
        )
        .await
        {
            Ok(shop_character_ids) => Json(Protocol::BuyResponse(
                GameUserInfo {
                    experience: game_user.experience,
                    health: game_user.health,
                    money: game_user.credits - character.cost as i32,
                    name: user.username.clone(),
                    avatar: game_user.avatar_id,
                },
                shop_service::get_shop(&shop_character_ids),
                character_service::get_board(&db, game_user.id)
                    .await
                    .unwrap(),
            )),
            Err(err) => {
                warn!("Could not upgrade character: {:?}", err);
                Json(Error::new_protocol_response(
                    Status::InternalServerError.code,
                    "Internal server error".to_string(),
                    Protocol::BuyRequest(buy_request.into_inner()),
                ))
            }
        }
    } else {
        if let Some(Some(existing_character)) =
            game_user_characters.0.get(buy_request.target_idx as usize)
        {
            if let Some((idx, _)) = game_user_characters
                .0
                .iter()
                .enumerate()
                .find(|(_, c)| c.is_none())
            {
                debug!("Moving character {:?} to slot {}", existing_character, idx);
                character_service::move_character(
                    &db,
                    (buy_request.target_idx, idx as u8),
                    existing_character.id,
                    &game_user_characters,
                )
                .await;
            } else {
                return Json(Error::new_protocol_response(
                    Status::UnprocessableEntity.code,
                    "Character slot alredy occupied".to_string(),
                    Protocol::BuyRequest(buy_request.into_inner()),
                ));
            }
        }

        if let Ok(shop_character_ids) = shop_service::buy_character(
            &db,
            &game_user,
            &character,
            shop,
            buy_request.character_idx,
            buy_request.target_idx,
        )
        .await
        {
            Json(Protocol::BuyResponse(
                GameUserInfo {
                    experience: game_user.experience,
                    health: game_user.health,
                    money: game_user.credits - character.cost as i32,
                    name: user.username.clone(),
                    avatar: game_user.avatar_id,
                },
                shop_service::get_shop(&shop_character_ids),
                character_service::get_board(&db, game_user.id)
                    .await
                    .unwrap(),
            ))
        } else {
            Json(Error::new_protocol_response(
                Status::InternalServerError.code,
                "Internal server error".to_string(),
                Protocol::BuyRequest(buy_request.into_inner()),
            ))
        }
    }
}
