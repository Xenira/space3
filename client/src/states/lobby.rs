use bevy::prelude::*;
use bevy_forms::{
    button::{self, ButtonClickEvent},
    list, text_input,
};

use surf::http::Method;

use crate::{
    cleanup_system, networking::networking_ressource::NetworkingRessource, AppState, Cleanup,
};

const STATE: AppState = AppState::Lobby;
pub(crate) struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(STATE).with_system(setup_ui))
            .add_system_set(text_input::add_to_system_set(
                SystemSet::on_update(STATE).with_system(button_click),
            ))
            .add_system_set(SystemSet::on_exit(STATE).with_system(cleanup_system::<Cleanup>));
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
                    "Lobby",
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
                .with_children(|parent| {
                    list::generate_list("list_users", None, parent);
                });
            parent
                .spawn_bundle(NodeBundle {
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    button::generate_button("Leave", "btn_leave", &asset_server, None, parent);
                    button::generate_button("Ready", "btn_ready", &asset_server, None, parent);
                    button::generate_button("Start", "btn_start", &asset_server, None, parent);
                });
        })
        .insert(Cleanup);
}

fn button_click(
    mut network: ResMut<NetworkingRessource>,
    mut ev_button_click: EventReader<ButtonClickEvent>,
) {
    for ev in ev_button_click.iter() {
        match ev.0.as_str() {
            "btn_leave" => network.request(Method::Delete, "lobbys"),
            _ => (),
        }
    }
}
