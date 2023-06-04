use bevy::prelude::*;
use protocol::protocol::GameResult;

use crate::{
    cleanup_system,
    components::{
        animation::{
            Animation, AnimationIndices, AnimationRepeatType, AnimationState, AnimationTimer,
            AnimationTransition, AnimationTransitionType,
        },
        hover::{BoundingBox, ClickEvent, Clickable, Hoverable},
    },
    prefabs::animation,
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::GameResult;

pub(crate) struct GameResultPlugin;

impl Plugin for GameResultPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(STATE)))
            .add_system(on_back_button_click.in_set(OnUpdate(STATE)))
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

#[derive(Resource, Debug)]
pub struct GameResultRes(pub GameResult);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    game_result: Res<GameResultRes>,
) {
    commands
        .spawn((
            Cleanup,
            SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            // Placement text
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    format!("Your achieved place {}", game_result.0.place).as_str(),
                    TextStyle {
                        font: asset_server.load("fonts/monogram-extended.ttf"),
                        font_size: 60.0,
                        color: Color::WHITE,
                    },
                ),
                ..Default::default()
            });

            // Rewards
            // TODO:Add rewards

            // God image
            // TODO: implement

            // Final board
            // TODO: implement

            // Back button
            let button = asset_server.load("textures/ui/expanding_frame.png");
            let button_atlas =
                TextureAtlas::from_grid(button, Vec2::new(64.0, 16.0), 20, 1, None, None);
            let button_atlas_handle = texture_atlases.add(button_atlas);

            let mut button_animation = Animation::default()
                .with_state(
                    AnimationState::new("init", AnimationIndices::from(0..14))
                        .with_repeat_type(AnimationRepeatType::Once)
                        .with_on_finish("idle"),
                )
                .with_state(
                    AnimationState::new("idle", AnimationIndices::from(14..14))
                        .with_repeat_type(AnimationRepeatType::Loop),
                )
                .with_global_transition(AnimationTransition {
                    name: "init".to_string(),
                    to_state: "idle".to_string(),
                    transition_type: AnimationTransitionType::Imediate,
                })
                .with_current_state("init");
            animation::add_hover_state(&mut button_animation, 14, 19);

            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: button_atlas_handle,
                        transform: Transform::from_translation(Vec3::new(0.0, -128.0, 0.0))
                            .with_scale(Vec3::splat(3.0)),
                        ..Default::default()
                    },
                    Hoverable::default(),
                    BoundingBox::from(Vec2::new(64.0, 16.0)),
                    Clickable,
                    button_animation,
                    AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                ))
                .with_children(|parent| {
                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            "Back",
                            TextStyle {
                                font: asset_server.load("fonts/monogram-extended.ttf"),
                                font_size: 16.0,
                                color: Color::WHITE,
                            },
                        ),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        ..Default::default()
                    });
                });
        });
}

fn on_back_button_click(
    mut ev_state: EventWriter<StateChangeEvent>,
    mut ev_click: EventReader<ClickEvent>,
) {
    for _ in ev_click.iter() {
        ev_state.send(StateChangeEvent(AppState::MenuMain));
    }
}
