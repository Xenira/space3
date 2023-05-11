use bevy::prelude::*;
use bevy_forms::{
    button::{self, ButtonClickEvent},
    form::{self, Form, FormError, FormMapping, FormValue, FromFormMapping, IntoFormMapping},
    text_input,
};
use protocol::protocol::{LobbyJoinRequest, Protocol};
use surf::http::Method;

use crate::{
    cleanup_system,
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::DIALOG_LOBBY_JOIN;
pub(crate) struct DialogLobbyJoinPlugin;

impl Plugin for DialogLobbyJoinPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Form<LobbyJoin>>()
            .add_system_set(SystemSet::on_enter(STATE).with_system(setup_ui))
            .add_system_set(text_input::add_to_system_set(
                SystemSet::on_update(STATE)
                    .with_system(button_click)
                    .with_system(on_login)
                    .with_system(form::on_change::<LobbyJoin>),
            ))
            .add_system_set(SystemSet::on_exit(STATE).with_system(cleanup_system::<Cleanup>));
    }
}

#[derive(Default)]
struct LobbyJoin(LobbyJoinRequest);

impl IntoFormMapping for LobbyJoin {
    fn into_mapping(self) -> FormMapping {
        let mut data: FormMapping = FormMapping::new();
        data.insert("name".to_string(), FormValue::String(self.0.name));
        data.insert(
            "passphrase".to_string(),
            FormValue::String(self.0.passphrase),
        );
        data
    }
}

impl FromFormMapping for LobbyJoin {
    fn from_mapping(form: &FormMapping) -> Result<Self, FormError> {
        let name = match form.get("name") {
            Some(FormValue::String(name)) => name.clone(),
            Some(actual) => {
                return Err(FormError::TypeMismatch {
                    expected: FormValue::String("name".to_string()),
                    got: actual.clone(),
                })
            }
            _ => return Err(FormError::FieldsMissing("name".to_string())),
        };
        let passphrase = match form.get("passphrase") {
            Some(FormValue::String(pw)) => pw.clone(),
            Some(actual) => {
                return Err(FormError::TypeMismatch {
                    expected: FormValue::String("passphrase".to_string()),
                    got: actual.clone(),
                })
            }
            _ => "".to_string(),
        };
        Ok(Self(LobbyJoinRequest { name, passphrase }))
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
                    "Join Lobby:",
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
                .with_children(|inputs| {
                    text_input::generate_input(
                        "name",
                        0,
                        "",
                        Some("Lobby-Name".to_string()),
                        None,
                        &asset_server,
                        None,
                        inputs,
                    );
                    text_input::generate_input(
                        "passphrase",
                        1,
                        "",
                        Some("Passphrase".to_string()),
                        Some("#".to_string()),
                        &asset_server,
                        None,
                        inputs,
                    );
                });
            parent
                .spawn_bundle(NodeBundle {
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    button::generate_button("Cancel", "btn_cancel", &asset_server, None, parent);
                    button::generate_button("Join", "btn_join", &asset_server, None, parent);
                });
        })
        .insert(Cleanup);
}

fn button_click(
    mut network: ResMut<NetworkingRessource>,
    form: Res<Form<LobbyJoin>>,
    mut ev_button_click: EventReader<ButtonClickEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    for ev in ev_button_click.iter() {
        debug!("Form mapping: {:?}", form.get_mapping());
        match ev.0.as_str() {
            "btn_join" => match form.get() {
                Ok(req) => network.request_data(Method::Put, "lobbys", &req.0),
                Err(err) => error!("Failed to send login request {:?}", err),
            },
            "btn_cancel" => ev_state_change.send(StateChangeEvent(AppState::MENU_MAIN)),
            _ => (),
        }
    }
}

fn on_login(
    mut commands: Commands,
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    for ev in ev_networking.iter() {
        if let Protocol::LOGIN_RESPONSE(login) = &ev.0 {
            commands.insert_resource(login.user.clone());

            ev_state_change.send(StateChangeEvent(AppState::LOBBY));
        }
    }
}
