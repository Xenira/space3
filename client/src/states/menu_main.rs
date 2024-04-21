use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};

use crate::{
    cleanup_system,
    modules::ui::{
        button::{ButtonClickEvent, ButtonProperties, ButtonRole},
        dialog::{ButtonGroup, Dialog},
    },
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::MenuMain;
pub(crate) struct MenuMainPlugin;

impl Plugin for MenuMainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(STATE), setup)
            .add_systems(Update, on_button_click.run_if(in_state(STATE)))
            .add_systems(OnExit(STATE), cleanup_system::<Cleanup>);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Dialog {
        title: Some("Main Menu".to_string()),
        buttons: ButtonGroup {
            buttons: vec![
                ButtonProperties {
                    text: "Play".to_string(),
                    role: ButtonRole::Primary,
                    id: Some("play".to_string()),
                },
                ButtonProperties {
                    text: "Sign Out".to_string(),
                    role: ButtonRole::Cancel,
                    id: Some("sign_out".to_string()),
                },
                ButtonProperties {
                    text: "Exit".to_string(),
                    role: ButtonRole::Cancel,
                    id: Some("exit".to_string()),
                },
            ],
            direction: FlexDirection::Column,
        },
        ..default()
    });
}

fn on_button_click(
    mut ev_state_change: EventWriter<StateChangeEvent>,
    mut ev_exit: EventWriter<AppExit>,
    mut ev_button_click: EventReader<ButtonClickEvent>,
) {
    for event in ev_button_click.read() {
        match event.button.id.as_deref() {
            Some("play") => {
                ev_state_change.send(StateChangeEvent(AppState::DialogLobbyJoin));
            }
            Some("sign_out") => {
                ev_state_change.send(StateChangeEvent(AppState::MenuLogin));
            }
            Some("exit") => {
                ev_exit.send(AppExit);
            }
            _ => {}
        }
    }
}
