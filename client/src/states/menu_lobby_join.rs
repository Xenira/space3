use crate::{
    cleanup_system,
    modules::ui::{
        button::{ButtonClickEvent, ButtonProperties, ButtonRole},
        dialog::{ButtonGroup, Dialog},
    },
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    states::menu_lobby::CurrentLobby,
    AppState, Cleanup, StateChangeEvent,
};
use bevy::prelude::*;
use bevy_ui_forms::prelude::*;
use protocol::protocol::{LobbyJoinRequest, Protocol};
use reqwest::Method;

const STATE: AppState = AppState::DialogLobbyJoin;
pub struct DialogLobbyJoinPlugin;

impl Plugin for DialogLobbyJoinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(STATE), setup)
            .add_systems(Update, (on_button_click, on_join).run_if(in_state(STATE)))
            .add_systems(OnExit(STATE), cleanup_system::<Cleanup>);
    }
}

#[derive(Resource)]
struct LobbyForm {
    lobby_name: Entity,
    passphrase: Entity,
}

fn setup(mut commands: Commands) {
    let lobby_name_input = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(200.0),
                    // height: Val::Px(50.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                ..default()
            },
            TextInputBundle::default()
                .with_text_style(TextStyle {
                    font_size: 22.0,
                    color: Color::BLACK,
                    ..default()
                })
                .with_placeholder("Lobby-Name", None)
                .with_active(true),
        ))
        .id();

    let passphrase_input = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(200.0),
                    // height: Val::Px(50.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                ..default()
            },
            TextInputBundle::default()
                .with_text_style(TextStyle {
                    font_size: 22.0,
                    color: Color::BLACK,
                    ..default()
                })
                .with_placeholder("Passphrase", None)
                .with_settings(TextInputSettings {
                    mask_character: Some('*'),
                    ..default()
                }),
        ))
        .id();

    let inputs = commands
        .spawn((NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            ..default()
        },))
        .id();

    commands.insert_resource(LobbyForm {
        lobby_name: lobby_name_input,
        passphrase: passphrase_input,
    });

    commands
        .entity(inputs)
        .add_child(lobby_name_input)
        .add_child(passphrase_input);

    commands.spawn((
        Dialog {
            title: Some("Join Lobby".to_string()),
            buttons: ButtonGroup {
                buttons: vec![
                    ButtonProperties {
                        text: "Back".to_string(),
                        role: ButtonRole::Cancel,
                        id: Some("back".to_string()),
                    },
                    ButtonProperties {
                        text: "Join".to_string(),
                        role: ButtonRole::Primary,
                        id: Some("join".to_string()),
                    },
                ],
                ..default()
            },
            content: Some(inputs),
            ..default()
        },
        Cleanup,
    ));
}

fn on_button_click(
    mut ev_state_change: EventWriter<StateChangeEvent>,
    mut ev_button_click: EventReader<ButtonClickEvent>,
    res_lobby_form: Res<LobbyForm>,
    q_input: Query<&TextInputValue>,
    mut network: ResMut<NetworkingRessource>,
) {
    for event in ev_button_click.read() {
        let lobby = LobbyJoinRequest {
            name: q_input.get(res_lobby_form.lobby_name).unwrap().0.clone(),
            passphrase: q_input.get(res_lobby_form.passphrase).unwrap().0.clone(),
        };
        match event.button.id.as_deref() {
            Some("back") => {
                ev_state_change.send(StateChangeEvent(AppState::MenuMain));
            }
            Some("join") => {
                network.request_data(Method::PUT, "lobbies", &lobby);
            }
            _ => {}
        }
    }
}

fn on_join(
    mut commands: Commands,
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    for ev in ev_networking.read() {
        if let Protocol::LobbyStatusResponse(lobby) = &ev.0 {
            debug!("Got lobby response {:?}", lobby);

            commands.insert_resource(CurrentLobby(lobby.clone()));

            ev_state_change.send(StateChangeEvent(AppState::Lobby));
        }
    }
}
