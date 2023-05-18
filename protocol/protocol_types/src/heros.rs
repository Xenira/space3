use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct God {
    pub id: i32,
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub pantheon: Cow<'static, str>,
}
