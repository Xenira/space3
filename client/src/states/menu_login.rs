use std::{collections::HashMap, time::Duration};

use bevy::{app::AppExit, prelude::*};
use protocol::protocol::{Credentials, Protocol};
use surf::http::Method;

use crate::{
    cleanup_system,
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    ui::{
        button::{self, ButtonClickEvent},
        form::{self, Form, FormError, FormMapping, FormValue, FromFormMapping, IntoFormMapping},
        text_input,
    },
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::MENU_LOGIN;
pub(crate) struct MenuLoginPlugin;

impl Plugin for MenuLoginPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Form<Credentials>>()
            .add_system_set(SystemSet::on_enter(STATE).with_system(setup_ui))
            .add_system_set(text_input::add_to_system_set(
                SystemSet::on_update(STATE)
                    .with_system(button_click)
                    .with_system(on_login)
                    .with_system(form::on_change::<Credentials>),
            ))
            .add_system_set(SystemSet::on_exit(STATE).with_system(cleanup_system::<Cleanup>));
    }
}

impl IntoFormMapping for Credentials {
    fn into_mapping(self) -> FormMapping {
        let mut data: FormMapping = FormMapping::new();
        data.insert("username".to_string(), FormValue::String(self.username));
        data.insert("password".to_string(), FormValue::String(self.password));
        data
    }
}

impl FromFormMapping for Credentials {
    fn from_mapping(form: &FormMapping) -> Result<Self, FormError> {
        let username = match form.get("username") {
            Some(FormValue::String(un)) => un.clone(),
            Some(actual) => {
                return Err(FormError::TypeMismatch {
                    expected: FormValue::String("username".to_string()),
                    got: actual.clone(),
                })
            }
            _ => return Err(FormError::FieldsMissing("username".to_string())),
        };
        let password = match form.get("password") {
            Some(FormValue::String(pw)) => pw.clone(),
            _ => return Err(FormError::FieldsMissing("password".to_string())),
        };
        Ok(Self { username, password })
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
                    "Login:",
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
                        "username",
                        0,
                        "",
                        Some("Username".to_string()),
                        None,
                        inputs,
                        &asset_server,
                    );
                    text_input::generate_input(
                        "password",
                        1,
                        "",
                        Some("Password".to_string()),
                        Some("*".to_string()),
                        inputs,
                        &asset_server,
                    );
                });
            parent
                .spawn_bundle(NodeBundle {
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    button::generate_button("Login", "btn_sign_in", parent, &asset_server);
                    button::generate_button("Register", "btn_register", parent, &asset_server);
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
            "btn_sign_in" => match form.get() {
                Ok(creds) => network.request_data(Method::Post, "users", &creds),
                Err(err) => error!("Failed to send login request {:?}", err),
            },
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

            ev_state_change.send(StateChangeEvent(AppState::MENU_MAIN));
        }
    }
}
