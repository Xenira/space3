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
                    let character = CHARACTERS[c.character_id as usize].clone();
                    CharacterInstance {
                        id: c.id,
                        character_id: c.character_id,
                        position: c.position,
                        attack: character.damage,
                        attack_bonus: c.attack_bonus,
                        defense: character.health,
                        defense_bonus: c.defense_bonus,
                        upgraded: c.upgraded,
                    }
                })
        })
        .collect::<Vec<_>>())
}
