use bevy::prelude::*;

pub(crate) mod character;
pub(crate) mod game_user_info;
pub(crate) mod god;
pub(crate) mod spell;
pub(crate) mod ui;

pub(crate) struct ModulesPlugin;

impl Plugin for ModulesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(character::CharacterPlugin)
            .add_plugins(game_user_info::GameUserInfoPlugin)
            .add_plugins(god::GodPlugin)
            .add_plugins(spell::SpellPlugin)
            .add_plugins(ui::UiModulesPlugin);
    }
}
