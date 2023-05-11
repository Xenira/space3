use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct God {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub pantheon: String,
}
