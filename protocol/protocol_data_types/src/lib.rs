use protocol_types::{character::CharacterUpgrade, heros::Pantheon, prelude::Ability};
use serde::{Deserialize, Serialize};

pub trait ToString {
    fn to_string(&self, idx: usize) -> String;
}

pub trait Named {
    fn get_name(&self) -> &str;
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GodJson {
    pub name: String,
    pub description: String,
    pub pantheon: Pantheon,
}

impl ToString for GodJson {
    fn to_string(&self, idx: usize) -> String {
        format!(
            "God {{ id: {}, name: Borrowed(\"{}\"), description: Borrowed(\"{}\"), pantheon: Pantheon::{:?} }}",
            idx, self.name, self.description, self.pantheon
        )
    }
}

impl Named for GodJson {
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CharacterJson {
    pub name: String,
    pub description: String,
    pub health: i32,
    pub attack: i32,
    pub cost: i32,
    pub upgrade: Option<CharacterUpgrade>,
    pub abilities: Vec<Ability>,
}

impl ToString for CharacterJson {
    fn to_string(&self, idx: usize) -> String {
        format!(
            "Character {{ id: {}, name: Borrowed(\"{}\"), description: Borrowed(\"{}\"), health: {}, attack: {}, cost: {}, upgrade: {}, abilities: vec![]}}",
            idx,
            self.name,
            self.description,
            self.health,
            self.attack,
            self.cost,
            if let Some(upgrade) = &self.upgrade { format!("Some(CharacterUpgrade {{ name: Borrowed(\"{}\"), attack: {}, health: {}, abilities: vec![] }})", upgrade.name, upgrade.attack, upgrade.health) } else { "None".to_string() }
        )
    }
}

impl Named for CharacterJson {
    fn get_name(&self) -> &str {
        &self.name
    }
}
