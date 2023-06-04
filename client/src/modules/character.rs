use bevy::prelude::*;
use protocol::{characters::get_characters, protocol::CharacterInstance};

use crate::{
    components::{
        animation::{
            AnimationIndices, AnimationRepeatType, AnimationState, AnimationTimer,
            AnimationTransition, AnimationTransitionType,
        },
        dragndrop::{DragEvent, Dragged},
        hover::HoverEvent,
        tooltip::SetTooltipEvent,
    },
    prefabs::animation,
    states::startup::{CharacterAssets, UiAssets},
    util::text::break_text,
    Cleanup,
};

const PORTRAIT_HEIGHT: f32 = 192.0;
const TOOLTIP_SCALE: f32 = 1.5;
const TOOLTIP_NAME_HEIGHT: f32 = 28.0 * TOOLTIP_SCALE;
const TOOLTIP_NAME_WIDTH: f32 = 150.0 * TOOLTIP_SCALE;
const TOOLTIP_DESCRIPTION_WIDTH: f32 = 250.0 * TOOLTIP_SCALE;
const TOOLTIP_DESCRIPTION_HEIGHT: f32 = 200.0 * TOOLTIP_SCALE;
const TOOLTIP_COLOR: Color = Color::rgba(0.75, 0.25, 0.75, 0.75);

pub(crate) struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((on_spawn, on_character_hover, on_character_drag));
    }
}

#[derive(Component, Debug)]
pub struct Character(pub CharacterInstance);

#[derive(Component, Debug)]
pub struct Health;

#[derive(Component, Debug)]
pub struct Attack;

fn on_spawn(
    mut commands: Commands,
    character_assets: Res<CharacterAssets>,
    ui_assets: Res<UiAssets>,
    q_added: Query<(&Character, Entity), Added<Character>>,
) {
    for (character, entity) in q_added.iter() {
        commands
            .entity(entity)
            .insert(Cleanup)
            .with_children(|parent| {
                spawn_character_portrait(
                    parent,
                    &character.0,
                    &character_assets,
                    &ui_assets,
                    false,
                );
            });
    }
}

fn spawn_character_portrait(
    parent: &mut ChildBuilder,
    character: &CharacterInstance,
    character_assets: &CharacterAssets,
    ui_assets: &UiAssets,
    full_info: bool,
) {
    let character_animation = animation::simple(0, 0)
        .with_state(
            AnimationState::new("die", AnimationIndices::new(0, 0))
                .with_repeat_type(AnimationRepeatType::Once)
                .with_fps(18.0),
        )
        .with_global_transition(AnimationTransition {
            name: "die".to_string(),
            to_state: "die".to_string(),
            transition_type: AnimationTransitionType::Imediate,
        });

    parent
        .spawn(SpriteBundle {
            texture: character_assets.character_frame.clone(),
            transform: Transform::from_scale(Vec3::splat(0.1))
                .with_translation(Vec3::new(0.0, 0.0, 5.0)),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: character_assets
                            .character_portraits
                            .get(&character.character_id)
                            .unwrap()
                            .clone(),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0))
                            .with_scale(Vec3::splat(1.0)),
                        ..Default::default()
                    },
                    character_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                ))
                .with_children(|parent| {
                    parent
                        .spawn(SpatialBundle {
                            transform: Transform::from_scale(Vec3::splat(6.0))
                                .with_translation(Vec3::new(0.0, 0.0, 5.0)),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            // Attack
                            parent
                                .spawn(SpriteBundle {
                                    texture: character_assets.attack_orb.clone(),
                                    transform: Transform::from_translation(Vec3::new(
                                        -24.0, -28.0, 0.0,
                                    ))
                                    .with_scale(Vec3::splat(0.75)),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(Text2dBundle {
                                        text: Text::from_section(
                                            (character.attack + character.attack_bonus).to_string(),
                                            TextStyle {
                                                font: ui_assets.font.clone(),
                                                font_size: 28.0,
                                                color: Color::WHITE,
                                            },
                                        ),
                                        transform: Transform::from_translation(Vec3::new(
                                            0.0, 0.0, 1.0,
                                        )),
                                        ..Default::default()
                                    });
                                });

                            // Health
                            parent
                                .spawn(SpriteBundle {
                                    texture: character_assets.health_orb.clone(),
                                    transform: Transform::from_translation(Vec3::new(
                                        24.0, -28.0, 0.0,
                                    ))
                                    .with_scale(Vec3::splat(0.75)),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(Text2dBundle {
                                        text: Text::from_section(
                                            (character.health + character.health_bonus).to_string(),
                                            TextStyle {
                                                font: ui_assets.font.clone(),
                                                font_size: 24.0,
                                                color: Color::WHITE,
                                            },
                                        ),
                                        transform: Transform::from_translation(Vec3::new(
                                            0.0, 0.0, 1.0,
                                        )),
                                        ..Default::default()
                                    });
                                });

                            if full_info {
                                let character = &get_characters()[character.character_id as usize];
                                parent
                                    .spawn(SpatialBundle {
                                        transform: Transform::from_scale(Vec3::splat(0.2)),
                                        ..Default::default()
                                    })
                                    .with_children(|parent| {
                                        // Name
                                        parent
                                            .spawn(SpriteBundle {
                                                sprite: Sprite {
                                                    color: TOOLTIP_COLOR,
                                                    custom_size: Some(Vec2::new(
                                                        TOOLTIP_NAME_WIDTH,
                                                        TOOLTIP_NAME_HEIGHT,
                                                    )),
                                                    ..default()
                                                },
                                                transform: Transform::from_translation(Vec3::new(
                                                    0.,
                                                    -PORTRAIT_HEIGHT,
                                                    5.,
                                                )),
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent.spawn(Text2dBundle {
                                                    text: Text::from_section(
                                                        character.name.clone(),
                                                        TextStyle {
                                                            font: ui_assets.font.clone(),
                                                            font_size: 36.0 * TOOLTIP_SCALE,
                                                            color: Color::WHITE,
                                                        },
                                                    ),
                                                    transform: Transform::from_translation(
                                                        Vec3::new(0.0, 0.0, 1.0),
                                                    ),
                                                    ..Default::default()
                                                });
                                            });

                                        // Description
                                        parent
                                            .spawn(SpriteBundle {
                                                sprite: Sprite {
                                                    color: TOOLTIP_COLOR,
                                                    custom_size: Some(Vec2::new(
                                                        TOOLTIP_DESCRIPTION_WIDTH,
                                                        TOOLTIP_DESCRIPTION_HEIGHT,
                                                    )),
                                                    ..default()
                                                },
                                                transform: Transform::from_translation(Vec3::new(
                                                    0.,
                                                    -PORTRAIT_HEIGHT
                                                        - TOOLTIP_NAME_HEIGHT
                                                        - TOOLTIP_DESCRIPTION_HEIGHT / 2.0,
                                                    5.,
                                                )),
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent.spawn(Text2dBundle {
                                                    text: Text::from_section(
                                                        break_text(
                                                            character.description.clone(),
                                                            TOOLTIP_DESCRIPTION_WIDTH,
                                                            24.0 * TOOLTIP_SCALE,
                                                            true,
                                                        ),
                                                        TextStyle {
                                                            font: ui_assets.font.clone(),
                                                            font_size: 24.0 * TOOLTIP_SCALE,
                                                            color: Color::WHITE,
                                                        },
                                                    ),
                                                    transform: Transform::from_translation(
                                                        Vec3::new(0.0, 0.0, 1.0),
                                                    ),
                                                    ..Default::default()
                                                });
                                            });
                                    });
                            }
                        });
                });
        });
}

fn on_character_hover(
    mut commands: Commands,
    mut ev_hover: EventReader<HoverEvent>,
    mut ev_tooltip: EventWriter<SetTooltipEvent>,
    q_character: Query<&Character, Without<Dragged>>,
    character_assets: Res<CharacterAssets>,
    ui_assets: Res<UiAssets>,
) {
    for HoverEvent(entity, is_hovered) in ev_hover.iter() {
        if let Ok(character) = q_character.get(*entity).map(|c| &c.0) {
            if *is_hovered {
                let tooltip = commands
                    .spawn((
                        SpatialBundle {
                            transform: Transform::from_scale(Vec3::splat(6.0))
                                .with_translation(Vec3::new(0.0, 150.0, 0.0)),
                            ..Default::default()
                        },
                        Cleanup,
                    ))
                    .with_children(|parent| {
                        spawn_character_portrait(
                            parent,
                            character,
                            &character_assets,
                            &ui_assets,
                            true,
                        );
                    })
                    .id();
                ev_tooltip.send(SetTooltipEvent(*entity, Some(tooltip)));
            } else {
                ev_tooltip.send(SetTooltipEvent(*entity, None));
            }
        }
    }
}

fn on_character_drag(
    mut ev_drag: EventReader<DragEvent>,
    mut ev_tooltip: EventWriter<SetTooltipEvent>,
) {
    for DragEvent(entity) in ev_drag.iter() {
        ev_tooltip.send(SetTooltipEvent(*entity, None));
    }
}
