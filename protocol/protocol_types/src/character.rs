use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Character {
    pub id: i32,
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub health: i32,
    pub damage: i32,
    pub cost: u8,
}
