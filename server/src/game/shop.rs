use super::SHOP_SIZE;
use protocol::{characters::CHARACTERS, protocol::CharacterInstance};
use rand::seq::SliceRandom;

#[derive(Debug, Default)]
pub struct Shop {
    pub characters: Vec<Option<CharacterInstance>>,
    pub locked: bool,
}

impl Shop {
    pub fn new() -> Self {
        Self {
            characters: Self::get_new_characters(SHOP_SIZE),
            locked: false,
        }
    }

    pub fn fill(&mut self) {
        // Remove all None values
        self.characters.retain(|c| c.is_some());

        // Fill the rest of the shop
        self.characters.append(&mut Self::get_new_characters(
            SHOP_SIZE - self.characters.len(),
        ));
        self.locked = false;
    }

    pub fn get_new_characters(count: usize) -> Vec<Option<CharacterInstance>> {
        CHARACTERS
            .choose_multiple(&mut rand::thread_rng(), count as usize)
            .cloned()
            .map(|c| Some(CharacterInstance::from(&c, false)))
            .collect::<Vec<_>>()
    }
}
