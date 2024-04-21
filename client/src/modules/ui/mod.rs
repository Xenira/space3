use bevy::prelude::*;

pub(crate) mod button;
pub(crate) mod dialog;

pub(crate) struct UiModulesPlugin;

impl Plugin for UiModulesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(dialog::DialogPlugin)
            .add_plugins(button::ButtonPlugin);
    }
}
