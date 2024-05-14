use bevy::prelude::*;
use bevy_ui_forms::prelude::*;

#[form_struct(actions = MainMenuActions)]
#[derive(Debug)]
pub struct MainMenu;

#[derive(FormActions, Event)]
#[form_action(form_type = MainMenu)]
pub enum MainMenuActions {
    #[form_action(text = "Start Game")]
    StartGame,
    Settings,
    About,
    #[form_action(text = "Sign Out")]
    SignOut,
    Exit,
}
