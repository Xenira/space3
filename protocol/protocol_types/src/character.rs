use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Character {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub health: i32,
    pub damage: i32,
    pub cost: i32,
}