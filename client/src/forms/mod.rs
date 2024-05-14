use bevy::app::{PluginGroup, PluginGroupBuilder};

pub(crate) mod login_form;
pub(crate) mod main_menu_form;

pub(crate) struct FormsPluginGroup;

impl PluginGroup for FormsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(login_form::LoginDataFormPlugin)
            .add(main_menu_form::MainMenuFormPlugin)
    }
}
