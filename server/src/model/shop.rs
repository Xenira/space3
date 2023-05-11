use crate::{model::game_users::GameUser, schema::shops, Database};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use protocol::{characters::CHARACTERS, protocol::Protocol};
use rand::seq::SliceRandom;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use super::game::Game;

#[derive(Queryable, Identifiable, Associations, Serialize, Deserialize, Clone, Debug)]
#[diesel(belongs_to(GameUser))]
pub struct Shop {
    pub id: i32,
    pub game_user_id: i32,
    pub character_ids: Vec<Option<i32>>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, Clone, Debug)]
#[table_name = "shops"]
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
#[table_name = "shops"]
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

#[get("/games/shop")]
pub async fn get_shop(db: crate::Database, game_user: GameUser) -> Json<Protocol> {
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
            .filter_map(|c| c)
            .map(|c| CHARACTERS[c as usize].clone())
            .collect::<Vec<_>>(),
    ))
}

#[post("/games/shop")]
pub async fn reroll_shop(db: Database, game_user: GameUser) -> Json<Protocol> {
    let shop = db
        .run(move |c| {
            let shop = shops::table
                .filter(shops::game_user_id.eq(game_user.id))
                .get_result::<Shop>(c)
                .unwrap();

            let new_shop = ShopUpdate::new();

            diesel::update(&shop).set(&new_shop).execute(c).unwrap();

            new_shop
        })
        .await;

    Json(Protocol::GameShopResponse(
        shop.character_ids
            .into_iter()
            .filter_map(|c| c)
            .map(|c| CHARACTERS[c as usize].clone())
            .collect::<Vec<_>>(),
    ))
}
