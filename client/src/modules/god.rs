use bevy::prelude::*;
use protocol::{protocol::GameOpponentInfo, protocol_types::character};

use crate::states::startup::{CharacterAssets, GodAssets, UiAssets};

pub(crate) struct GodPlugin;

impl Plugin for GodPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_spawn);
    }
}

#[derive(Component, Debug)]
pub struct God(pub GameOpponentInfo);

fn on_spawn(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    god_assets: Res<GodAssets>,
    character_assets: Res<CharacterAssets>,
    q_added: Query<(&God, Entity), Added<God>>,
) {
    for (god, entity) in q_added.iter() {
        // Not used as hover animation is used for next opponent instead
        // let mut frame_animation = animation::simple(0, 0);
        // animation::add_hover_state(&mut frame_animation, 0, 17);

        commands.entity(entity).with_children(|parent| {
            parent
                .spawn((SpriteSheetBundle {
                    texture_atlas: god_assets.god_frame.clone(),
                    sprite: TextureAtlasSprite::new(if god.0.is_next_opponent { 17 } else { 0 }),
                    transform: Transform::from_scale(Vec3::splat(1.0))
                        .with_translation(Vec3::new(0.0, 0.0, 5.0)),
                    ..Default::default()
                },))
                .with_children(|parent| {
                    // God Portrait
                    parent.spawn(SpriteSheetBundle {
                        texture_atlas: god_assets
                            .god_portraits
                            .get(&god.0.character_id)
                            .unwrap()
                            .clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0))
                            .with_scale(Vec3::splat(0.25)),
                        ..Default::default()
                    });
                    // Health
                    parent
                        .spawn(SpriteBundle {
                            texture: character_assets.health_orb.clone(),
                            transform: Transform::from_translation(Vec3::new(24.0, -28.0, 0.0))
                                .with_scale(Vec3::splat(0.75)),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent.spawn(Text2dBundle {
                                text: Text::from_section(
                                    (god.0.health).to_string(),
                                    TextStyle {
                                        font: ui_assets.font.clone(),
                                        font_size: 24.0,
                                        color: Color::WHITE,
                                    },
                                ),
                                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                                ..Default::default()
                            });
                        });
                    parent
                        .spawn(SpriteBundle {
                            texture: god_assets.lvl_orb.clone(),
                            transform: Transform::from_translation(Vec3::new(-24.0, -28.0, 0.0))
                                .with_scale(Vec3::splat(0.75)),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent.spawn(Text2dBundle {
                                text: Text::from_section(
                                    (god.0.experience).to_string(),
                                    TextStyle {
                                        font: ui_assets.font.clone(),
                                        font_size: 24.0,
                                        color: Color::WHITE,
                                    },
                                ),
                                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                                ..Default::default()
                            });
                        });
                });
        });
    }
}
