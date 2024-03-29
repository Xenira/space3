use super::SHOP_SIZE;
use protocol::{characters::get_characters, protocol::CharacterInstance};
use rand::seq::SliceRandom;

#[derive(Debug, Default, Clone)]
pub struct Shop {
    pub characters: Vec<Option<CharacterInstance>>,
    pub locked: bool,
}

impl Shop {
    pub fn new(lvl: u8) -> Self {
        Self {
            characters: Self::get_new_characters(SHOP_SIZE, lvl),
            locked: false,
        }
    }

    pub fn fill(&mut self, lvl: u8) {
        // Remove all None values
        self.characters.retain(|c| c.is_some());

        // Fill the rest of the shop
        self.characters.append(&mut Self::get_new_characters(
            SHOP_SIZE - self.characters.len(),
            lvl,
        ));
        self.locked = false;
    }

    pub fn get_new_characters(count: usize, lvl: u8) -> Vec<Option<CharacterInstance>> {
        get_characters()
            .iter()
            .filter(|c| c.cost <= lvl)
            .collect::<Vec<_>>()
            .choose_multiple(&mut rand::thread_rng(), count)
            .cloned()
            .map(|c| Some(CharacterInstance::from(&c, false)))
            .collect::<Vec<_>>()
    }
}
