use crate::game::{game_instance::GameInstance, game_instance_player::GameInstancePlayer};
use protocol::protocol::CharacterInstance;
use rand::{rngs::StdRng, Rng};
use rand_core::{OsRng, SeedableRng};

pub async fn perform_bot_turns(game: &mut GameInstance) -> Result<(), diesel::result::Error> {
    let mut rng = rand::rngs::StdRng::from_seed(OsRng.gen());

    for bot in game.players.iter_mut().filter(|p| p.user_id.is_none()) {
        perform_bot_turn(bot, &mut rng).await;
    }

    Ok(())
}

pub async fn perform_bot_turn(bot: &mut GameInstancePlayer, rng: &mut StdRng) {
    for _ in 0..20 {
        let buyable = bot
            .shop
            .characters
            .iter()
            .filter_map(|c| c.as_ref())
            .filter(|c| c.cost as u16 <= bot.money)
            .collect::<Vec<_>>();

        // Reroll
        if buyable.is_empty() {
            if bot.reroll().is_err() {
                break;
            }
            continue;
        }

        let free_index = bot.get_free_board_index();

        // Check if free board space
        if free_index.is_none() {
            // Sell a character
            if let Some(sell_idx) = find_character_to_sell(bot.board.clone().to_vec()) {
                bot.sell(sell_idx);
                continue;
            } else {
                break;
            }
        }

        // Buy a character
        if let Some(character) = find_character_to_buy(buyable, bot.board.clone().to_vec(), rng) {
            bot.buy(character, free_index.unwrap());
            continue;
        }

        if bot.money == 0 {
            break;
        }
    }
}

fn find_character_to_sell(characters: Vec<Option<CharacterInstance>>) -> Option<usize> {
    let mut characters = characters
        .into_iter()
        .enumerate()
        .filter(|(_, c)| c.is_some())
        .map(|(i, c)| (i, c.unwrap()))
        .collect::<Vec<_>>();
    characters.sort_by_key(|(_, c)| c.attack + c.attack_bonus);

    characters.pop().map(|(i, _)| i)
}

fn find_character_to_buy(
    buyable: Vec<&CharacterInstance>,
    characters: Vec<Option<CharacterInstance>>,
    rng: &mut StdRng,
) -> Option<usize> {
    let idx = buyable
        .iter()
        .enumerate()
        .find(|(_, c)| {
            characters
                .iter()
                .filter_map(|b| b.as_ref())
                .any(|b| b.character_id == c.character_id)
        })
        .map(|(i, _)| i);

    idx.or_else(|| Some(rng.gen_range(0..buyable.len())))
}
