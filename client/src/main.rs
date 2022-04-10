use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::{StandardMaterial, PointLightBundle},
    prelude::*,
    DefaultPlugins, log::{LogSettings, Level}, hierarchy::{DespawnRecursiveExt}, math::Vec3,
};
use bevy_vox::*;

use crate::{ui::button::{self, ButtonClickEvent}, states::{game_states}, components::timer};

mod states;
mod ui;
mod util;
mod components;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    STARTUP,
    MENU_MAIN,
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

    app.add_state(AppState::MENU_MAIN)
        .insert_resource(LogSettings { level: Level::DEBUG, ..Default::default()})
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(VoxPlugin)
        .add_event::<ButtonClickEvent>()
        .add_event::<StateChangeEvent>()
        .add_startup_system(setup)
        .add_system(button::button_system)
        .add_system(state_change_handler)
        .add_plugin(timer::TimerPlugin)
        .add_plugins(game_states::GameStatesPluginGroup);
    
    debug!("Starting app");
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
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

fn state_change_handler(mut app_state: ResMut<State<AppState>>,
    mut ev_state_change: EventReader<StateChangeEvent>) {
    for ev in ev_state_change.iter() {
        debug!("State change {:?}", ev);
        app_state.overwrite_set(ev.0.clone());
    }
}

pub fn cleanup_system<T: Component>(
    mut commands: Commands,
    q: Query<Entity, With<T>>,
) {
    debug!("[CLEANUP] Removing entities");
    for e in q.iter() {
        debug!("[CLEANUP] Despawning {:?}", e);
        commands.entity(e).despawn_recursive();
    }
}