use crate::{
    model::{game_user_characters::NewGameUserCharacter, game_users::GameUser, shop::Shop},
    schema::{game_user_characters, game_users, shops},
    Database,
};
use diesel::{delete, dsl::sum, insert_into, prelude::*, update};
use protocol::{
    characters::CHARACTERS, protocol::CharacterInstance, protocol_types::character::Character,
};
use rocket::log::private::debug;

pub async fn upgrade_character(
    db: &Database,
    game_user: &GameUser,
    character: &Character,
    mut shop: Shop,
    shop_idx: u8,
    target_idx: u8,
) -> Result<Vec<Option<i32>>, diesel::result::Error> {
    debug!("upgrade_character {:?} {:?}", character, shop);

    let character_id = character.id;
    let price = character.cost as i32;
    let game_user_id = game_user.id;
    let shop_id = shop.id;
    shop.character_ids[shop_idx as usize] = None;
    db.run(move |c| {
        c.transaction(move |c| {
            let (attack_bonus, health_bonus) = delete(game_user_characters::table)
                .filter(
                    game_user_characters::game_user_id
                        .eq(game_user_id)
                        .and(game_user_characters::character_id.eq(character_id))
                        .and(game_user_characters::upgraded.eq(false)),
                )
                .returning((
                    game_user_characters::attack_bonus,
                    game_user_characters::defense_bonus,
                ))
                .get_results::<(i32, i32)>(c)?
                .iter()
                .fold((0, 0), |(c_attack, c_health), (n_attack, n_health)| {
                    (c_attack + n_attack, c_health + n_health)
                });

            debug!(
                "attack_bonus: {:?}, defense_bonus: {:?}",
                attack_bonus, health_bonus
            );

            insert_into(game_user_characters::table)
                .values(&NewGameUserCharacter {
                    game_user_id,
                    character_id,
                    position: target_idx as i32,
                    upgraded: true,
                    attack_bonus,
                    defense_bonus: health_bonus,
                })
                .execute(c)?;

            update(shops::table)
                .filter(shops::id.eq(shop_id))
                .set(shops::character_ids.eq(shop.character_ids.clone()))
                .execute(c)?;

            update(game_users::table)
                .filter(game_users::id.eq(game_user_id))
                .set(game_users::credits.eq(game_users::credits - price))
                .execute(c)?;
            QueryResult::Ok(shop.character_ids.clone())
        })
    })
    .await
}

pub async fn buy_character(
    db: &Database,
    game_user: &GameUser,
    character: &Character,
    mut shop: Shop,
    shop_idx: u8,
    target_idx: u8,
) -> Result<Vec<Option<i32>>, diesel::result::Error> {
    let character_id = character.id;
    let price = character.cost as i32;
    let game_user_id = game_user.id;
    let shop_id = shop.id;
    shop.character_ids[shop_idx as usize] = None;
    db.run(move |c| {
        c.build_transaction().run(move |c| {
            insert_into(game_user_characters::table)
                .values(&NewGameUserCharacter {
                    game_user_id,
                    character_id,
                    position: target_idx as i32,
                    upgraded: false,
                    attack_bonus: 0,
                    defense_bonus: 0,
                })
                .execute(c)?;

            update(shops::table)
                .filter(shops::id.eq(shop_id))
                .set(shops::character_ids.eq(shop.character_ids.clone()))
                .execute(c)?;

            update(game_users::table)
                .filter(game_users::id.eq(game_user_id))
                .set(game_users::credits.eq(game_users::credits - price))
                .execute(c)?;

            QueryResult::Ok(shop.character_ids.clone())
        })
    })
    .await
    .map_err(|e| {
        debug!("buy_character error: {:?}", e);
        e
    })
}

pub fn get_shop(shop_character_ids: &Vec<Option<i32>>) -> Vec<Option<(u8, CharacterInstance)>> {
    shop_character_ids
        .iter()
        .map(|c| {
            c.and_then(|c| {
                Some((
                    CHARACTERS[c as usize].cost,
                    CharacterInstance::from(&CHARACTERS[c as usize], false),
                ))
            })
        })
        .collect::<Vec<_>>()
}
