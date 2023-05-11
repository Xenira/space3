use bevy::{core::Stopwatch, prelude::*};

use crate::{
    cleanup_system,
    components::timer::StopwatchComponent,
    ui::button::{self, ButtonClickEvent},
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::GAME_SEARCH;

pub(crate) struct GameSearchPlugin;

impl Plugin for GameSearchPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(STATE).with_system(setup))
            .add_system_set(
                SystemSet::on_update(STATE)
                    .with_system(button_click)
                    .with_system(timer),
            )
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
                            "Searching...",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
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
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 40.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                                Default::default(),
                            ),
                            ..Default::default()
                        })
                        .insert(StopwatchComponent {
                            time: Stopwatch::new(),
                        });
                });
            button::generate_button("Cancel", "btn_cancel", parent, &asset_server);
        })
        .insert(Cleanup);
}

fn button_click(
    mut ev_button_click: EventReader<ButtonClickEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    for ev in ev_button_click.iter() {
        match ev.0.as_str() {
            "btn_cancel" => ev_state_change.send(StateChangeEvent(AppState::MENU_MAIN)),
            _ => (),
        }
    }
}

fn timer(
    mut ev_state_change: EventWriter<StateChangeEvent>,
    mut timer: Query<(&StopwatchComponent, &mut Text), With<Text>>,
) {
    for (watch, mut text) in timer.iter_mut() {
        text.sections[0].value = (watch.time.elapsed_secs() as u32).to_string();
        if watch.time.elapsed_secs() >= 5.0 {
            ev_state_change.send(StateChangeEvent(AppState::GAME_COMMANDER_SELECTION));
        }
    }
    // let mut text = text_query.get_mut(watch.).unwrap();
}