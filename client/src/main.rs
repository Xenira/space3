#[cfg(not(target_family = "wasm"))]
extern crate dotenv;
use crate::{
    components::{timer::TimerUi, ComponentsPlugin}, forms::main_menu_form::MainMenuForm, modules::game_user_info::GameUserRes, networking::networking_plugin::NetworkingPlugin, states::{
        game_combat::BattleRes, game_commander_selection::GameCommanderSelection,
        game_result::GameResultRes, game_states,
    }
};
use bevy::{
    asset::AssetMetaCheck,
    diagnostic::EntityCountDiagnosticsPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    render::view::RenderLayers,
    window::Cursor,
};
use bevy_ecss::{EcssPlugin, RegisterComponentSelector, StyleSheet};
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts, EguiPlugin, EguiStartupSet,
};
use bevy_ui_forms::BevyUiFormsPlugins;
use chrono::{DateTime, Utc};
use components::on_screen_log::{LogEntry, LogLevel};
#[cfg(not(target_family = "wasm"))]
use dotenv::dotenv;
use networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource};
use protocol::{
    gods::get_gods,
    protocol::{Credentials, Protocol, Turn},
};
use reqwest::Method;
use states::startup::startup::UiRoot;
use std::env;

mod components;
mod forms;
mod modules;
mod networking;
mod prefabs;
mod states;
mod util;

pub(crate) const UI_LAYER: u8 = 20;
pub(crate) const CURSOR_LAYER: u8 = 21;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Startup,
    Loading,
    MenuLogin,
    MenuSettings,
    MenuSetDisplayName,
    MenuMain,
    DialogLobbyJoin,
    Lobby,
    GameSearch,
    GameCommanderSelection,
    GameShop,
    GameBattle,
    GameResult,
}

#[derive(Debug, Event)]
pub struct StateChangeEvent(AppState);

#[derive(Component, Debug)]
pub struct Cleanup;

#[derive(Component, Debug)]
pub struct Id(String);

fn main() {
    debug!("Generating app");
    let mut app = App::new();
    let log_level = Level::DEBUG;

    #[cfg(not(target_family = "wasm"))]
    if let Err(err) = dotenv() {
        warn!("Failed to read dotenv: {}", err);
    }

    #[cfg(not(target_family = "wasm"))]
    let base_url = env::var("BASE_URL").expect("BASE_URL not supplied");
    #[cfg(target_family = "wasm")]
    let base_url = wasm_get_base_url()
        .expect("Base url could not be resolved")
        .to_string();

    let mut cursor = Cursor::default();
    cursor.visible = false;

    let default_plugins = DefaultPlugins
        .set(LogPlugin {
            level: log_level,
            ..Default::default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "<name>".to_string(),
                resizable: true,
                cursor,
                ..Default::default()
            }),
            ..Default::default()
        })
        .set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..default()
        });

    app.init_state::<AppState>()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(default_plugins) // .set(ImagePlugin::default_nearest()),
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EntityCountDiagnosticsPlugin)
        .add_plugins(EguiPlugin)
        .add_event::<StateChangeEvent>()
        .add_plugins(NetworkingPlugin(base_url))
        .add_systems(Startup, setup.after(EguiStartupSet::InitContexts))
        .add_systems(
            Update,
            (state_change_handler, networking_handler, refresh_css),
        )
        .add_plugins(ComponentsPlugin)
        .add_plugins(modules::ModulesPlugin)
        .add_plugins(game_states::GameStatesPluginGroup)
        .add_plugins(EcssPlugin::with_hot_reload())
        .register_component_selector::<MainMenuForm>("main-menu")
        .add_plugins(BevyUiFormsPlugins)
        .add_plugins(forms::FormsPluginGroup);

    debug!("Starting app");
    app.run();
}

fn refresh_css(
    q_node_added: Query<Entity, Added<Style>>,
    mut q_ui_root: Query<&mut StyleSheet, With<UiRoot>>,
) {
    for _ in q_node_added.iter() {
        for mut style in &mut q_ui_root {
            style.refresh();
        }
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct UiCamera;

pub fn setup(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut networking: ResMut<NetworkingRessource>,
) {
    commands
        // camera
        .spawn((Camera2dBundle::default(), MainCamera));

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: UI_LAYER as isize,
                ..default()
            },
            ..default()
        },
        UiCamera,
        RenderLayers::from_layers(&[UI_LAYER]),
    ));
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: CURSOR_LAYER as isize,
                ..default()
            },
            ..default()
        },
        RenderLayers::from_layers(&[CURSOR_LAYER]),
    ));

    contexts.ctx_mut().set_visuals(egui::Visuals {
        panel_fill: Color32::TRANSPARENT,
        ..Default::default()
    });

    if let Ok(user) = env::var("USER") {
        if let Ok(pass) = env::var("PASS") {
            warn!(
                "Logging in as {} from env. Except during development you prob. shouldn't do this!",
                user,
            );
            networking.request_data(
                Method::POST,
                "users",
                &Credentials {
                    username: user,
                    password: pass,
                },
            );
        }
    }
}

#[cfg(target_family = "wasm")]
fn wasm_get_base_url() -> Option<String> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    document
        .base_uri()
        .expect("should have a base URI on document")
}

fn networking_handler(
    mut commands: Commands,
    mut ev_net: EventReader<NetworkingEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
    mut ev_log: EventWriter<LogEntry>,
) {
    for ev in ev_net.read() {
        debug!("[NET] {:?}", ev.0);
        ev_log.send(LogEntry {
            text: format!("[NET] {:?}", ev.0),
            lvl: LogLevel::Info,
            ..Default::default()
        });

        match &ev.0 {
            Protocol::LobbyLeaveResponse => {
                ev_state_change.send(StateChangeEvent(AppState::MenuMain));
            }
            Protocol::LobbyStatusResponse(lobby) => {
                if let Some(timer) = lobby.start_at {
                    commands.insert_resource(TimerUi(Some(Timer::from_seconds(
                        timer.signed_duration_since(Utc::now()).num_seconds() as f32,
                        TimerMode::Once,
                    ))));
                }
                ev_state_change.send(StateChangeEvent(AppState::Lobby));
            }
            Protocol::GameStartResponse(gods) => {
                commands.insert_resource(TimerUi(Some(Timer::from_seconds(30.0, TimerMode::Once))));
                commands.insert_resource(GameCommanderSelection(
                    gods.iter()
                        .map(|g| get_gods()[*g as usize].clone())
                        .collect::<Vec<_>>(),
                ));
                ev_state_change.send(StateChangeEvent(AppState::GameCommanderSelection));
            }
            Protocol::GameUpdateResponse(update) => {
                let timer: DateTime<Utc> = update.turn.into();

                commands.insert_resource(TimerUi(Some(Timer::from_seconds(
                    timer.signed_duration_since(Utc::now()).num_seconds() as f32,
                    TimerMode::Once,
                ))));

                if let Turn::Shop(_, _) = update.turn {
                    ev_state_change.send(StateChangeEvent(AppState::GameShop));
                };
            }
            Protocol::GameUserInfoResponse(user_info) => {
                commands.insert_resource(GameUserRes(user_info.clone()));
            }
            Protocol::GameBattleResponse(battle) => {
                commands.insert_resource(BattleRes(battle.clone()));
                ev_state_change.send(StateChangeEvent(AppState::GameBattle));
            }
            Protocol::GameEndResponse(result) => {
                commands.remove_resource::<GameUserRes>();
                commands.insert_resource(GameResultRes(result.clone()));
                ev_state_change.send(StateChangeEvent(AppState::GameResult));
            }
            Protocol::NetworkingError(e) => {
                if e.status == 401 {
                    ev_state_change.send(StateChangeEvent(AppState::MenuLogin));
                }
            }
            _ => {}
        };
    }
}

fn state_change_handler(
    current_state: Res<State<AppState>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut ev_state_change: EventReader<StateChangeEvent>,
) {
    for ev in ev_state_change.read() {
        if *current_state.get() == ev.0 {
            warn!("State change {:?} is already active", ev);
            continue;
        }

        debug!("State change {:?}", ev);
        match ev.0 {
            AppState::DialogLobbyJoin => app_state.set(AppState::DialogLobbyJoin),
            _ => app_state.set(ev.0.clone()),
        }
    }
}

pub fn cleanup_system<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    debug!("[CLEANUP] Removing entities");
    for e in q.iter() {
        debug!("[CLEANUP] Despawning {:?}", e);
        commands.entity(e).despawn_recursive();
    }
}
