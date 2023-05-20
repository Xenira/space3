use crate::{
    model::game_user_characters::GameUserCharacter, schema::game_user_characters, Database,
};
use diesel::prelude::*;
use protocol::{
    characters::CHARACTERS, protocol::CharacterInstance, protocol_types::character::Character,
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
