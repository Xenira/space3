use bevy::{app::AppExit, prelude::*};

use crate::{
    cleanup_system,
    ui::button::{self, ButtonClickEvent},
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::MENU_MAIN;
pub(crate) struct MenuMainPlugin;

impl Plugin for MenuMainPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(STATE).with_system(setup))
            .add_system_set(SystemSet::on_update(STATE).with_system(button_click))
            .add_system_set(SystemSet::on_exit(STATE).with_system(cleanup_system::<Cleanup>));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // root node
    let ui = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // bevy logo (flex center)
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        position_type: PositionType::Absolute,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    // bevy logo (image)
                    parent.spawn_bundle(ImageBundle {
                        style: Style {
                            size: Size::new(Val::Px(500.0), Val::Auto),
                            ..Default::default()
                        },
                        image: asset_server.load("branding/bevy_logo_dark_big.png").into(),
                        ..Default::default()
                    });
                });
            button::generate_button("Play", "btn_play", parent, &asset_server);
            button::generate_button("Exit", "btn_exit", parent, &asset_server);
        })
        .insert(Cleanup);
}

fn button_click(
    mut ev_button_click: EventReader<ButtonClickEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
    mut ev_exit: EventWriter<AppExit>,
) {
    for ev in ev_button_click.iter() {
        match ev.0.as_str() {
            "btn_play" => ev_state_change.send(StateChangeEvent(AppState::GAME_SEARCH)),
            "btn_exit" => ev_exit.send(AppExit),
            _ => (),
        }
    }
}
