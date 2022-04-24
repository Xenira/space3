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
use networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource};

use crate::{components::timer, networking::networking::NetworkingPlugin, states::game_states};

mod components;
mod networking;
mod states;
mod util;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    STARTUP,
    MENU_LOGIN,
    MENU_MAIN,
    DIALOG_LOBBY_JOIN,
    LOBBY,
    GAME_SEARCH,
    GAME_COMMANDER_SELECTION,
    GAME_SHOP,
    GAME_BATTLE,
    GAME_RESULT,
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

    app.add_state(AppState::MENU_LOGIN)
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
        .add_plugin(NetworkingPlugin(
            "http://localhost:8000/api/v1/".to_string(),
        ))
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
        match ev.0 {
            AppState::DIALOG_LOBBY_JOIN => app_state.push(AppState::DIALOG_LOBBY_JOIN),
            _ => app_state.overwrite_set(ev.0.clone()),
        };
    }
}

pub fn cleanup_system<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    debug!("[CLEANUP] Removing entities");
    for e in q.iter() {
        debug!("[CLEANUP] Despawning {:?}", e);
        commands.entity(e).despawn_recursive();
    }
}
