pub(crate) mod data_manager;
pub(crate) mod game_instance;
pub(crate) mod game_instance_player;
pub(crate) mod shop;

pub(crate) const BOARD_SIZE: usize = 12;
pub(crate) const SHOP_SIZE: usize = 5;
pub(crate) const MAX_LVL: u8 = 10;
pub(crate) const EXP_PER_LVL: u8 = 3;

pub(crate) const COMBAT_DURATION_MULTIPLIER: f64 = 1.1;
pub(crate) const DEFAULT_COMBAT_DURATION: i64 = 5;

pub(crate) const START_HEALTH: i16 = 10;
pub(crate) const START_EXP: u8 = 5;
pub(crate) const START_MONEY: u16 = 2;
