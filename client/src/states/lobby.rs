use bevy::{app::AppExit, prelude::*};
use bevy_forms::{
    button::{self, ButtonClickEvent},
    form::Form,
    text_input,
};
use protocol::protocol::{Credentials, Protocol};

use crate::{
    cleanup_system,
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::LOBBY;
pub(crate) struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(STATE).with_system(setup_ui))
            .add_system_set(text_input::add_to_system_set(
                SystemSet::on_update(STATE)
                    .with_system(button_click)
                    .with_system(on_login),
            ))
            .add_system_set(SystemSet::on_exit(STATE).with_system(cleanup_system::<Cleanup>));
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_grow: 1.0,
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // bevy logo (flex center)
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Lobby",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::ColumnReverse,
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|inputs| {});
            parent
                .spawn_bundle(NodeBundle {
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    button::generate_button("Leave", "btn_leave", &asset_server, None, parent);
                    button::generate_button("Ready", "btn_ready", &asset_server, None, parent);
                    button::generate_button("Start", "btn_start", &asset_server, None, parent);
                });
        })
        .insert(Cleanup);
}

fn button_click(
    mut network: ResMut<NetworkingRessource>,
    form: Res<Form<Credentials>>,
    mut ev_button_click: EventReader<ButtonClickEvent>,
    mut ev_exit: EventWriter<AppExit>,
) {
    for ev in ev_button_click.iter() {
        debug!("Form mapping: {:?}", form.get_mapping());
        match ev.0.as_str() {
            "btn_exit" => ev_exit.send(AppExit),
            _ => (),
        }
    }
}

fn on_login(
    mut commands: Commands,
    mut network: ResMut<NetworkingRessource>,
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    for ev in ev_networking.iter() {
        if let Protocol::LOGIN_RESPONSE(login) = &ev.0 {
            network.client = network
                .client
                .config()
                .clone()
                .add_header("x-api-key", login.key.clone())
                .unwrap()
                .try_into()
                .unwrap();
            commands.insert_resource(login.user.clone());

            ev_state_change.send(StateChangeEvent(AppState::DIALOG_LOBBY_JOIN));
        }
    }
}
