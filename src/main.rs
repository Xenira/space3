use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::StandardMaterial,
    prelude::{App, Assets, Commands, Mesh, ResMut, SystemSet},
    DefaultPlugins,
};

use crate::ui::button;

mod main_menu;
mod ui;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MENU_MAIN,
    GAME_SHOP,
    GAME_BATTLE,
}

fn main() {
    println!("Hello, world!");
    App::new()
        .add_state(AppState::MENU_MAIN)
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::on_enter(AppState::MENU_MAIN)
                .with_system(main_menu::setup)
        )
        .add_system_set(
            SystemSet::on_update(AppState::MENU_MAIN)
                .with_system(main_menu::mouse_scroll)
        )
        .add_system(button::button_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
}
