use bevy::{app::AppExit, prelude::*};

use crate::{
    cleanup_system,
    modules::ui::{
        button::{ButtonClickEvent, ButtonProperties, ButtonRole},
        dialog::{ButtonGroup, Dialog},
    },
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::MenuSettings;
pub(crate) struct MenuSettingsPlugin;

impl Plugin for MenuSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(STATE), setup)
            .add_systems(Update, on_button_click.run_if(in_state(STATE)))
            .add_systems(OnExit(STATE), cleanup_system::<Cleanup>);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Dialog {
        title: Some("Settings".to_string()),
        text: Some("Settings are not implemented yet".to_string()),
        buttons: ButtonGroup {
            buttons: vec![ButtonProperties {
                text: "Back".to_string(),
                role: ButtonRole::Cancel,
                id: Some("back".to_string()),
            }],
            ..default()
        },
        ..default()
    });
}

fn on_button_click(
    mut ev_state_change: EventWriter<StateChangeEvent>,
    mut ev_button_click: EventReader<ButtonClickEvent>,
) {
    for event in ev_button_click.read() {
        match event.button.id.as_deref() {
            Some("back") => {
                ev_state_change.send(StateChangeEvent(AppState::MenuMain));
            }
            _ => {}
        }
    }
}
