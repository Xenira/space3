#[cfg(not(target_family = "wasm"))]
extern crate dotenv;
use components::on_screen_log::{LogEntry, LogLevel};
#[cfg(not(target_family = "wasm"))]
use dotenv::dotenv;
use reqwest::Method;

use crate::{
    components::{timer::TimerUi, ComponentsPlugin},
    modules::game_user_info::GameUserRes,
    networking::networking::NetworkingPlugin,
    states::{
        game_combat::BattleRes, game_commander_selection::GameCommanderSelection,
        game_result::GameResultRes, game_states,
    },
};
use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, LogDiagnosticsPlugin},
    hierarchy::DespawnRecursiveExt,
    log::{Level, LogPlugin},
    math::Vec3,
    pbr::PointLightBundle,
    prelude::*,
    window::Cursor,
    winit::WinitPlugin,
};
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts, EguiPlugin,
};
use chrono::{DateTime, Utc};
use networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource};
use protocol::{
    gods::GODS,
    protocol::{Credentials, Protocol, Turn},
};
use std::env;

mod components;
mod modules;
mod networking;
mod prefabs;
mod states;
mod util;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Startup,
    MenuLogin,
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

#[derive(Debug)]
pub struct StateChangeEvent(AppState);

#[derive(Component, Debug)]
pub struct Cleanup;

#[derive(Component, Debug)]
pub struct Id(String);

fn main() {
    debug!("Generating app");
    let mut app = App::new();

    #[cfg(not(target_family = "wasm"))]
    if let Err(err) = dotenv() {
        warn!("Failed to read dotenv: {}", err);
    }

    #[cfg(not(target_family = "wasm"))]
    let base_url = env::var("BASE_URL").expect("BASE_URL not supplied");
    #[cfg(target_family = "wasm")]
    let base_url = env!("BASE_URL", "BASE_URL needs to be set for wasm builds").to_string();

    let mut cursor = Cursor::default();
    cursor.visible = false;

    let default_plugins = DefaultPlugins
        .set(LogPlugin {
            level: Level::DEBUG,
            ..Default::default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "<name>".to_string(),
                resizable: true,
                cursor,
                fit_canvas_to_parent: true,
                ..Default::default()
            }),
            ..Default::default()
        });

    app.add_state::<AppState>()
        .add_plugins(default_plugins) // .set(ImagePlugin::default_nearest()),
        .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        .add_event::<StateChangeEvent>()
        .add_plugin(NetworkingPlugin(format!("{}/api/v1/", base_url)))
        .add_startup_system(setup)
        .add_system(state_change_handler)
        .add_system(networking_handler)
        .add_plugin(ComponentsPlugin)
        .add_plugin(modules::ModulesPlugin)
        .add_plugins(game_states::GameStatesPluginGroup);

    debug!("Starting app");
    app.run();
}

#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut networking: ResMut<NetworkingRessource>,
) {
    commands
        // light
        .spawn(PointLightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
            ..Default::default()
        });
    commands
        // camera
        .spawn((Camera2dBundle::default(), MainCamera));

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

fn networking_handler(
    mut commands: Commands,
    mut ev_net: EventReader<NetworkingEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
    mut ev_log: EventWriter<LogEntry>,
) {
    for ev in ev_net.iter() {
        debug!("[NET] {:?}", ev.0);
        ev_log.send(LogEntry {
            text: format!("[NET] {:?}", ev.0),
            lvl: LogLevel::Info,
            ..Default::default()
        });

        match &ev.0 {
            Protocol::LobbyLeaveResponse => {
                ev_state_change.send(StateChangeEvent(AppState::MenuMain))
            }
            Protocol::LobbyStatusResponse(lobby) => {
                if let Some(timer) = lobby.start_at {
                    commands.insert_resource(TimerUi(Some(Timer::from_seconds(
                        timer.signed_duration_since(Utc::now()).num_seconds() as f32,
                        TimerMode::Once,
                    ))));
                }
                ev_state_change.send(StateChangeEvent(AppState::Lobby))
            }
            Protocol::GameStartResponse(gods) => {
                commands.insert_resource(TimerUi(Some(Timer::from_seconds(30.0, TimerMode::Once))));
                commands.insert_resource(GameCommanderSelection(
                    gods.iter()
                        .map(|g| GODS[*g as usize].clone())
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

                match update.turn {
                    Turn::Combat(_, _) => (),
                    Turn::Shop(_, _) => ev_state_change.send(StateChangeEvent(AppState::GameShop)),
                }
            }
            Protocol::GameUserInfoResponse(user_info) => {
                commands.insert_resource(GameUserRes(user_info.clone()));
            }
            Protocol::GameBattleResponse(battle) => {
                commands.insert_resource(BattleRes(battle.clone()));
                ev_state_change.send(StateChangeEvent(AppState::GameBattle));
            }
            Protocol::GameEndResponse(result) => {
                commands.insert_resource(GameResultRes(result.clone()));
                ev_state_change.send(StateChangeEvent(AppState::GameResult))
            }
            Protocol::NetworkingError(e) => {
                if e.status == 401 {
                    ev_state_change.send(StateChangeEvent(AppState::MenuLogin))
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
    for ev in ev_state_change.iter() {
        if current_state.0 == ev.0 {
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
