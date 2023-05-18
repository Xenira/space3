use bevy::{prelude::*, transform::commands};

pub(crate) mod character;
pub(crate) mod game_user_info;

pub(crate) struct ModulesPlugin;

impl Plugin for ModulesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(character::CharacterPlugin)
            .add_plugin(game_user_info::GameUserInfoPlugin);
    }
}
