use crate::{
    cleanup_system,
    components::anchors::{AnchorType, Anchors},
    modules::ui::{
        button::{ButtonClickEvent, ButtonProperties, ButtonRole},
        dialog::{ButtonGroup, Dialog},
    },
    networking::{
        networking_events::NetworkingEvent, networking_ressource::NetworkingRessource,
        polling::PollingStatus,
    },
    prefabs::ui::timer::TimerTextBundle,
    AppState, Cleanup, StateChangeEvent,
};
use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};
use bevy_not_so_simple_text_input::{TextInputBundle, TextInputSettings, TextInputValue};
use protocol::protocol::{Credentials, Protocol, UserData};
use reqwest::{header::HeaderValue, Method};

const STATE: AppState = AppState::MenuLogin;
pub(crate) struct MenuLoginPlugin;

impl Plugin for MenuLoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(STATE), logout)
            .add_systems(Update, (on_login, on_button_click).run_if(in_state(STATE)))
            .add_systems(OnExit(STATE), cleanup_system::<Cleanup>);
    }
}

#[derive(Resource)]
pub struct User(pub UserData);

#[derive(Resource)]
pub struct LoginForm {
    pub username: Entity,
    pub password: Entity,
}

fn logout(
    mut commands: Commands,
    mut ev_polling_status: EventWriter<PollingStatus>,
    asset_server: Res<AssetServer>,
    res_anchors: Res<Anchors>,
) {
    debug!("Logout start");
    commands.remove_resource::<User>();
    ev_polling_status.send(PollingStatus::Stop);

    let username_input = commands
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
                .with_placeholder("Username", None)
                .with_active(true),
        ))
        .id();

    let password_input = commands
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
                .with_placeholder("Password", None)
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

    commands.insert_resource(LoginForm {
        username: username_input,
        password: password_input,
    });

    commands
        .entity(inputs)
        .add_child(username_input)
        .add_child(password_input);

    commands
        .entity(res_anchors.get(AnchorType::TOP_RIGHT).unwrap())
        .with_children(|parent| {
            parent.spawn((TimerTextBundle::new(
                &asset_server,
                Transform::from_translation(Vec3::new(-30.0, -15.0, 100.0)),
            ),));
            parent.spawn((
                Dialog {
                    title: Some("Welcome".to_string()),
                    buttons: ButtonGroup {
                        buttons: vec![
                            ButtonProperties {
                                text: "Register".to_string(),
                                role: ButtonRole::Secondary,
                                id: Some("register".to_string()),
                            },
                            ButtonProperties {
                                text: "Login".to_string(),
                                role: ButtonRole::Primary,
                                id: Some("login".to_string()),
                            },
                        ],
                        ..default()
                    },
                    content: Some(inputs),
                    ..default()
                },
                Cleanup,
            ));
        });

    debug!("Logout end")
}

fn on_button_click(
    mut network: ResMut<NetworkingRessource>,
    mut ev_button_click: EventReader<ButtonClickEvent>,
    res_form: Res<LoginForm>,
    q_text_input: Query<&TextInputValue>,
) {
    for ev in ev_button_click.read() {
        if let Some(id) = ev.button.id.as_deref() {
            let username = q_text_input.get(res_form.username).unwrap().0.clone();
            let password = q_text_input.get(res_form.password).unwrap().0.clone();
            let credentials = Credentials { username, password };
            match id {
                "login" => {
                    network.request_data(Method::POST, "users", &credentials);
                }
                "register" => {
                    network.request_data(Method::PUT, "users", &credentials);
                }
                _ => {}
            }
        }
    }
}

fn on_login(
    mut commands: Commands,
    mut network: ResMut<NetworkingRessource>,
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_polling_status: EventWriter<PollingStatus>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    for ev in ev_networking.read() {
        if let Protocol::LoginResponse(login) = &ev.0 {
            network.headers.insert(
                "x-api-key",
                HeaderValue::from_str(login.key.as_str()).unwrap(),
            );

            commands.insert_resource(User(login.user.clone()));
            debug!("Logged in as {}", login.user.username);

            ev_polling_status.send(PollingStatus::Start);
            if login.user.display_name.is_none() {
                ev_state_change.send(StateChangeEvent(AppState::MenuSetDisplayName));
            } else {
                ev_state_change.send(StateChangeEvent(AppState::MenuMain));
            }
        }
    }
}
