use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct God {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub pantheon: Pantheon,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum Pantheon {
    #[default]
    Greek,
    Norse,
    Egyptian,
    Hindu,
}

impl Pantheon {
    // could be generated by macro
    pub const VARIANTS: &'static [Pantheon] =
        &[Self::Greek, Self::Norse, Self::Egyptian, Self::Hindu];
}

impl Display for Pantheon {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{self:?}")
    }
}

impl From<&str> for Pantheon {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "greek" => Self::Greek,
            "norse" => Self::Norse,
            "egyptian" => Self::Egyptian,
            "hindu" => Self::Hindu,
            _ => panic!("Invalid pantheon: {}", s),
        }
    }
}
