#[cfg(not(target_family = "wasm"))]
extern crate dotenv;
#[cfg(not(target_family = "wasm"))]
use dotenv::dotenv;

use std::env;

use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, LogDiagnosticsPlugin},
    hierarchy::DespawnRecursiveExt,
    log::{Level, LogSettings},
    math::Vec3,
    pbr::PointLightBundle,
    prelude::*,
    DefaultPlugins,
};
use bevy_forms::{
    button::{self, ButtonClickEvent},
    form::FormPluginGroup,
};
use bevy_vox::*;
use networking::networking_events::NetworkingEvent;

use crate::{components::timer, networking::networking::NetworkingPlugin, states::game_states};

mod components;
mod networking;
mod states;
mod util;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    _Startup,
    MenuLogin,
    MenuMain,
    DialogLobbyJoin,
    Lobby,
    GameSearch,
    GameCommanderSelection,
    GameShop,
    _GameBattle,
    _GameResult,
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

    app.add_state(AppState::MenuLogin)
        .insert_resource(LogSettings {
            level: Level::DEBUG,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(VoxPlugin)
        .add_event::<ButtonClickEvent>()
        .add_event::<StateChangeEvent>()
        .add_plugin(NetworkingPlugin(format!("{}/api/v1/", base_url)))
        .add_startup_system(setup)
        .add_system(button::button_system)
        .add_system(state_change_handler)
        .add_system(networking_handler)
        .add_plugin(timer::TimerPlugin)
        .add_plugins(FormPluginGroup)
        .add_plugins(game_states::GameStatesPluginGroup);

    debug!("Starting app");
    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        // light
        .spawn_bundle(PointLightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
            ..Default::default()
        });
    commands
        // camera
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 100.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..Default::default()
        });
}

fn networking_handler(mut ev_net: EventReader<NetworkingEvent>) {
    for ev in ev_net.iter() {
        debug!("[NET] {:?}", ev.0)
    }
}

fn state_change_handler(
    mut app_state: ResMut<State<AppState>>,
    mut ev_state_change: EventReader<StateChangeEvent>,
) {
    for ev in ev_state_change.iter() {
        debug!("State change {:?}", ev);
        if let Err(err) = match ev.0 {
            AppState::DialogLobbyJoin => app_state.push(AppState::DialogLobbyJoin),
            _ => app_state.overwrite_set(ev.0.clone()),
        } {
            error!("Failed to change state {:?}", err);
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
