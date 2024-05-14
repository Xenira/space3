use bevy::{app::AppExit, prelude::*};
use bevy_ecss::Class;

use crate::{
    cleanup_system, forms::main_menu_form::MainMenuActions, modules::ui::dialog::Dialog, AppState,
    Cleanup, StateChangeEvent,
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
    let form = commands
        .spawn((
            crate::forms::main_menu_form::MainMenuForm,
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            Cleanup,
        ))
        .id();

    commands.spawn((
        Dialog {
            title: Some("Main Menu".to_string()),
            content: Some(form),
            ..default()
        },
        Class::new("main-menu"),
    ));
}

fn on_button_click(
    mut ev_state_change: EventWriter<StateChangeEvent>,
    mut ev_exit: EventWriter<AppExit>,
    mut ev_button_click: EventReader<MainMenuActions>,
) {
    for event in ev_button_click.read() {
        match event {
            MainMenuActions::StartGame => {
                ev_state_change.send(StateChangeEvent(AppState::DialogLobbyJoin));
            }
            MainMenuActions::Settings => {
                ev_state_change.send(StateChangeEvent(AppState::MenuSettings));
            }
            MainMenuActions::SignOut => {
                ev_state_change.send(StateChangeEvent(AppState::MenuLogin));
            }
            MainMenuActions::Exit => {
                ev_exit.send(AppExit);
            }
            _ => {}
        }
    }
}
