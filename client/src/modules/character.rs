use bevy::prelude::*;
use protocol::protocol::CharacterInstance;

use crate::{
    components::animation::{
        AnimationIndices, AnimationRepeatType, AnimationState, AnimationTimer, AnimationTransition,
        AnimationTransitionType,
    },
    prefabs::animation,
};

pub(crate) struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_spawn);
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
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    q_added: Query<(&Character, Entity), Added<Character>>,
) {
    for (character, entity) in q_added.iter() {
        let character_fallback = asset_server.load(format!(
            "generated/characters/{}.png",
            character.0.character_id
        ));
        let character_atlas = TextureAtlas::from_grid(
            character_fallback,
            Vec2::new(256.0, 256.0),
            1,
            1,
            None,
            None,
        );
        let character_atlas_handle = texture_atlases.add(character_atlas);

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

        commands.entity(entity).with_children(|parent| {
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: character_atlas_handle,
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0))
                            .with_scale(Vec3::splat(0.25)),
                        ..Default::default()
                    },
                    character_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                ))
                .with_children(|parent| {
                    parent
                        .spawn(SpatialBundle {
                            transform: Transform::from_scale(Vec3::splat(4.0))
                                .with_translation(Vec3::new(0.0, 0.0, 5.0)),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            // Attack
                            parent
                                .spawn(SpriteBundle {
                                    texture: asset_server.load("textures/ui/attack_orb.png"),
                                    transform: Transform::from_translation(Vec3::new(
                                        -24.0, -28.0, 0.0,
                                    ))
                                    .with_scale(Vec3::splat(0.75)),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(Text2dBundle {
                                        text: Text::from_section(
                                            (character.0.attack + character.0.attack_bonus)
                                                .to_string(),
                                            TextStyle {
                                                font: asset_server
                                                    .load("fonts/monogram-extended.ttf"),
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
                                    texture: asset_server.load("textures/ui/health_orb.png"),
                                    transform: Transform::from_translation(Vec3::new(
                                        24.0, -28.0, 0.0,
                                    ))
                                    .with_scale(Vec3::splat(0.75)),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(Text2dBundle {
                                        text: Text::from_section(
                                            (character.0.defense + character.0.defense_bonus)
                                                .to_string(),
                                            TextStyle {
                                                font: asset_server
                                                    .load("fonts/monogram-extended.ttf"),
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
                        });
                });
        });
    }
}
