use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Character {
    pub id: i32,
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub health: i32,
    pub attack: i32,
    pub cost: u8,
    pub upgrade: Option<CharacterUpgrade>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CharacterUpgrade {
    pub name: Cow<'static, str>,
    pub attack: i32,
    pub health: i32,
}
