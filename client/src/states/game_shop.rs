use std::time::Duration;

use bevy::prelude::*;
use bevy_forms::button::{self, ButtonClickEvent};

use crate::{
    cleanup_system, components::timer::TimerComponent, AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::GAME_SHOP;

pub(crate) struct GameShopPlugin;

impl Plugin for GameShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(STATE).with_system(setup))
            // .add_system_set(
            //     SystemSet::on_update(STATE)
            //         .with_system(button_click)
            //         .with_system(timer),
            // )
            .add_system_set(SystemSet::on_exit(STATE).with_system(cleanup_system::<Cleanup>));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // root node
    commands
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
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Select your Commander:",
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
                        .spawn_bundle(TextBundle {
                            text: Text::with_section(
                                "",
                                TextStyle {
                                    font: asset_server.load("fonts/monogram-extended.ttf"),
                                    font_size: 40.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                                Default::default(),
                            ),
                            ..Default::default()
                        })
                        .insert(TimerComponent {
                            time: Timer::new(Duration::from_secs(30), false),
                        });
                });
            button::generate_button("Select", "btn_select", &asset_server, None, parent);
        })
        .insert(Cleanup);
}

fn button_click(
    mut ev_button_click: EventReader<ButtonClickEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    for ev in ev_button_click.iter() {
        match ev.0.as_str() {
            "btn_select" => ev_state_change.send(StateChangeEvent(AppState::GAME_SHOP)),
            _ => (),
        }
    }
}

fn timer(
    mut ev_state_change: EventWriter<StateChangeEvent>,
    mut timer: Query<(&TimerComponent, &mut Text), With<Text>>,
) {
    for (watch, mut text) in timer.iter_mut() {
        if watch.time.finished() {
            ev_state_change.send(StateChangeEvent(AppState::GAME_SHOP));
        } else if watch.time.percent() >= 0.75 {
            text.sections[0].value = ((watch.time.duration().as_secs_f32()
                - watch.time.elapsed_secs()) as u32)
                .to_string();
        }
    }
}
