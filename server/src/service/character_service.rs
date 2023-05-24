use crate::{
    model::game_user_characters::{GameUserCharacter, GameUserCharacters},
    schema::game_user_characters,
    Database,
};
use diesel::{prelude::*, update};
use protocol::{
    characters::CHARACTERS,
    protocol::CharacterInstance,
    protocol_types::character::{self, Character},
};

pub async fn get_board(
    db: &Database,
    game_user_id: i32,
) -> Result<Vec<Option<CharacterInstance>>, diesel::result::Error> {
    let game_user_characters = db
        .run(move |con| {
            game_user_characters::table
                .filter(game_user_characters::game_user_id.eq(game_user_id))
                .load::<GameUserCharacter>(con)
        })
        .await?;

    let character_idx = 0..12;
    Ok(character_idx
        .map(|i| {
            game_user_characters
                .iter()
                .find(|c| c.position == i)
                .map(|c| {
                    CharacterInstance::from(&CHARACTERS[c.character_id as usize], c.upgraded)
                        .with_id(c.id)
                        .with_position(c.position)
                })
        })
        .collect::<Vec<_>>())
}

pub async fn move_character(
    db: &Database,
    (character_idx, target_idx): (u8, u8),
    source_character_id: i32,
    game_user_characters: &GameUserCharacters,
) {
    if character_idx != target_idx {
        let db_result =
            if let Some(target_character) = game_user_characters.0[target_idx as usize].clone() {
                // Swap characters
                let target_character_id = target_character.id;
                db.run(move |con| {
                    con.build_transaction().deferrable().run(|con| {
                        update(game_user_characters::table)
                            .filter(game_user_characters::id.eq(source_character_id))
                            .set(game_user_characters::position.eq(target_idx as i32))
                            .execute(con)?;

                        update(game_user_characters::table)
                            .filter(game_user_characters::id.eq(target_character_id))
                            .set(game_user_characters::position.eq(character_idx as i32))
                            .execute(con)?;

                        QueryResult::Ok(())
                    })
                })
                .await
            } else {
                // Move character to empty slot
                db.run(move |con| {
                    update(game_user_characters::table)
                        .filter(game_user_characters::id.eq(source_character_id))
                        .set(game_user_characters::position.eq(target_idx as i32))
                        .execute(con)?;

                    QueryResult::Ok(())
                })
                .await
            };

        if let Err(db_err) = db_result {
            warn!("Could not move character: {}", db_err);
        }
    } else {
        debug!("Target and source index are the same. Noop.");
    }
}
