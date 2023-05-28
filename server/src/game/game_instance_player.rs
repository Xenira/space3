use super::{shop::Shop, BOARD_SIZE, EXP_PER_LVL, MAX_LVL, START_EXP, START_HEALTH, START_MONEY};
use protocol::{
    characters::CHARACTERS,
    protocol::{CharacterInstance, GameOpponentInfo},
    protocol_types::prelude::God,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct GameInstancePlayer {
    pub id: Uuid,
    pub user_id: Option<i32>,
    pub display_name: String,
    pub board: [Option<CharacterInstance>; BOARD_SIZE],
    pub god: Option<God>,
    pub god_choices: [i32; 4],
    pub shop: Shop,
    pub health: i16,
    pub money: u16,
    pub experience: u8,
    pub placement: Option<u8>,
}

impl std::default::Default for GameInstancePlayer {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: None,
            display_name: String::new(),
            board: Default::default(),
            god: None,
            god_choices: Default::default(),
            shop: Default::default(),
            health: START_HEALTH,
            money: START_MONEY,
            experience: START_EXP,
            placement: None,
        }
    }
}

impl GameInstancePlayer {
    pub fn new(user_id: Option<i32>, display_name: String, god_choices: [i32; 4]) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            display_name,
            god_choices,
            ..Default::default()
        }
    }

    pub fn with_god(mut self, god: God) -> Self {
        self.god = Some(god);
        self
    }

    pub fn generate_shop(&mut self) {
        if self.shop.locked {
            self.shop.fill();
        } else {
            self.shop = Shop::new();
        }
    }

    pub fn reroll(&mut self) -> Result<(), ()> {
        if self.money < 1 {
            // Not enough money
            return Err(());
        }

        self.money -= 1;
        self.shop = Shop::new();
        Ok(())
    }

    pub fn buy(&mut self, shop_idx: usize, board_idx: usize) -> Result<(), ()> {
        let Some(shop_character) = self.shop.characters.get(shop_idx) else {
            // Invalid shop index
            return Err(());
        };

        let Some(mut shop_character) = shop_character.clone() else {
            // Character is not in shop
            return Err(());
        };

        let cost = shop_character.cost as u16;
        if self.money < cost as u16 {
            // Not enough money
            return Err(());
        }

        let mut upgradeable = self.get_upgradeable(shop_character.character_id);
        if upgradeable.len() == 2 {
            // Character is an upgrade
            upgradeable.push(shop_character);
            shop_character = self.upgrade(upgradeable)?;
        }

        let free_index = self.get_free_board_index();

        let Some(board_character) = self.board.get_mut(board_idx) else {
            // Invalid board index
            return Err(());
        };

        if let Some(board_character) = board_character {
            // Board is occupied at index
            let Some(free_index) = free_index else {
                // Board is full
                return Err(());
            };

            self.board[free_index] = Some(board_character.clone());
            self.board[board_idx] = Some(shop_character);

            self.money -= cost as u16;
            *self.shop.characters.get_mut(shop_idx).unwrap() = None;
        } else {
            // Board is empty at index
            self.money -= cost as u16;
            *board_character = Some(shop_character);
            *self.shop.characters.get_mut(shop_idx).unwrap() = None;
        }

        Ok(())
    }

    pub fn sell(&mut self, character_idx: usize) -> Result<(), ()> {
        if character_idx > self.board.len() {
            // Invalid board index
            return Err(());
        }

        if let Some(_) = self.board.get(character_idx).unwrap() {
            self.money += 1;
            self.board[character_idx] = None;
            Ok(())
        } else {
            // Board is empty at index
            return Err(());
        }
    }

    pub fn get_upgradeable(&self, character_id: i32) -> Vec<CharacterInstance> {
        self.board
            .iter()
            .filter_map(|c| c.clone())
            .filter(|c| c.character_id == character_id && !c.upgraded)
            .collect::<Vec<_>>()
    }

    pub fn upgrade(&mut self, characters: Vec<CharacterInstance>) -> Result<CharacterInstance, ()> {
        // 3 characters must be provided to upgrade
        if characters.len() != 3 {
            return Err(());
        }

        // All characters must be the same and not be upgraded
        let character = characters.first().unwrap();
        let character_id = character.character_id;
        if characters
            .iter()
            .any(|c| c.upgraded || c.character_id != character.character_id)
        {
            return Err(());
        }

        // Calculate bonuses
        let (attack_bonus, health_bonus) = self
            .board
            .iter_mut()
            .filter_map(|c| c.as_ref())
            .map(|c| (c.attack_bonus, c.health_bonus))
            .fold((0, 0), |(c_attack, c_health), (n_attack, n_health)| {
                (c_attack + n_attack, c_health + n_health)
            });

        // Remove characters from board
        for character in characters {
            self.board
                .iter_mut()
                .filter(|c| c.is_some())
                .filter(|c| c.as_ref().unwrap().id == character.id)
                .for_each(|c| *c = None);
        }

        Ok(
            CharacterInstance::from(&CHARACTERS[character_id as usize], true)
                .with_attack_bonus(attack_bonus)
                .with_health_bonus(health_bonus),
        )
    }

    pub fn move_character(&mut self, from_idx: usize, to_idx: usize) -> Result<(), ()> {
        let board_len = self.board.len();
        if from_idx >= board_len || to_idx >= board_len {
            // Invalid board index
            return Err(());
        }

        self.board.swap(from_idx, to_idx);

        Ok(())
    }

    pub fn get_free_board_index(&self) -> Option<usize> {
        self.board.iter().position(|c| c.is_none())
    }

    pub fn get_lvl(&self) -> u8 {
        (self.experience / EXP_PER_LVL).min(MAX_LVL)
    }

    pub fn opponent_info(&self, is_next_opponent: bool) -> GameOpponentInfo {
        GameOpponentInfo {
            name: self.display_name.clone(),
            experience: self.experience,
            health: self.health,
            character_id: self.god.as_ref().map_or(0, |g| g.id),
            is_next_opponent,
        }
    }
}
