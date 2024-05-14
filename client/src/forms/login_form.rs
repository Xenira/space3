use bevy::prelude::*;
use bevy_ui_forms::prelude::*;
use protocol::protocol::Credentials;

#[form_struct(submit = "Login", cancel = "Exit")]
#[derive(Debug, Clone)]
pub struct LoginData {
    #[form_field(active)]
    #[text_box(placeholder = "Username")]
    pub username: String,
    #[text_box(placeholder = "Password", mask = '*')]
    pub password: String,
}

impl Into<Credentials> for LoginData {
    fn into(self) -> Credentials {
        Credentials {
            username: self.username,
            password: self.password,
        }
    }
}
