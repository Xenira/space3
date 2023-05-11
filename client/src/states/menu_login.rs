use bevy::{app::AppExit, prelude::*};
use bevy_forms::{
    button::{self, ButtonClickEvent},
    form::{self, Form, FormError, FormMapping, FormValue, FromFormMapping, IntoFormMapping},
    text_input,
};
use protocol::protocol::{Credentials, Protocol};
use surf::http::Method;

use crate::{
    cleanup_system,
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::MENU_LOGIN;
pub(crate) struct MenuLoginPlugin;

impl Plugin for MenuLoginPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Form<LoginCredentials>>()
            .add_system_set(SystemSet::on_enter(STATE).with_system(setup_ui))
            .add_system_set(text_input::add_to_system_set(
                SystemSet::on_update(STATE)
                    .with_system(button_click)
                    .with_system(on_login)
                    .with_system(form::on_change::<LoginCredentials>),
            ))
            .add_system_set(SystemSet::on_exit(STATE).with_system(cleanup_system::<Cleanup>));
    }
}

#[derive(Default)]
struct LoginCredentials(Credentials);

impl IntoFormMapping for LoginCredentials {
    fn into_mapping(self) -> FormMapping {
        let mut data: FormMapping = FormMapping::new();
        data.insert("username".to_string(), FormValue::String(self.0.username));
        data.insert("password".to_string(), FormValue::String(self.0.password));
        data
    }
}

impl FromFormMapping for LoginCredentials {
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
        Ok(Self(Credentials { username, password }))
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
                        font: asset_server.load("fonts/monogram-extended.ttf"),
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
                        &asset_server,
                        None,
                        inputs,
                    );
                    text_input::generate_input(
                        "password",
                        1,
                        "",
                        Some("Password".to_string()),
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
                    button::generate_button("Login", "btn_sign_in", &asset_server, None, parent);
                    button::generate_button(
                        "Register",
                        "btn_register",
                        &asset_server,
                        None,
                        parent,
                    );
                });
        })
        .insert(Cleanup);
}

fn button_click(
    mut network: ResMut<NetworkingRessource>,
    form: Res<Form<LoginCredentials>>,
    mut ev_button_click: EventReader<ButtonClickEvent>,
    mut ev_exit: EventWriter<AppExit>,
) {
    for ev in ev_button_click.iter() {
        debug!("Form mapping: {:?}", form.get_mapping());
        match ev.0.as_str() {
            "btn_sign_in" => match form.get() {
                Ok(creds) => network.request_data(Method::Post, "users", &creds.0),
                Err(err) => error!("Failed to send login request {:?}", err),
            },
            "btn_register" => match form.get() {
                Ok(creds) => network.request_data(Method::Put, "users", &creds.0),
                Err(err) => error!("Failed to send registration request {:?}", err),
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
