use bevy::prelude::*;
use protocol::protocol::GameOpponentInfo;

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
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    q_added: Query<(&God, Entity), Added<God>>,
) {
    for (god, entity) in q_added.iter() {
        let god_fallback = asset_server.load(format!("generated/gods/{}.png", god.0.character_id));
        let god_atlas =
            TextureAtlas::from_grid(god_fallback, Vec2::new(256.0, 256.0), 1, 1, None, None);
        let god_atlas_handle = texture_atlases.add(god_atlas);

        let god_frame = asset_server.load("textures/ui/god_frame2.png");
        let god_frame_atlas =
            TextureAtlas::from_grid(god_frame, Vec2::new(64.0, 64.0), 18, 1, None, None);
        let god_frame_atlas_handle = texture_atlases.add(god_frame_atlas);

        // Not used as hover animation is used for next opponent instead
        // let mut frame_animation = animation::simple(0, 0);
        // animation::add_hover_state(&mut frame_animation, 0, 17);

        commands.entity(entity).with_children(|parent| {
            parent
                .spawn((SpriteSheetBundle {
                    texture_atlas: god_frame_atlas_handle.clone(),
                    sprite: TextureAtlasSprite::new(if god.0.is_next_opponent { 17 } else { 0 }),
                    transform: Transform::from_scale(Vec3::splat(1.0))
                        .with_translation(Vec3::new(0.0, 0.0, 5.0)),
                    ..Default::default()
                },))
                .with_children(|parent| {
                    // God Portrait
                    parent.spawn(SpriteSheetBundle {
                        texture_atlas: god_atlas_handle,
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0))
                            .with_scale(Vec3::splat(0.25)),
                        ..Default::default()
                    });
                    // Health
                    parent
                        .spawn(SpriteBundle {
                            texture: asset_server.load("textures/ui/health_orb.png"),
                            transform: Transform::from_translation(Vec3::new(24.0, -28.0, 0.0))
                                .with_scale(Vec3::splat(0.75)),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent.spawn(Text2dBundle {
                                text: Text::from_section(
                                    (god.0.health).to_string(),
                                    TextStyle {
                                        font: asset_server.load("fonts/monogram-extended.ttf"),
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
                            texture: asset_server.load("textures/ui/lvl_orb.png"),
                            transform: Transform::from_translation(Vec3::new(-24.0, -28.0, 0.0))
                                .with_scale(Vec3::splat(0.75)),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent.spawn(Text2dBundle {
                                text: Text::from_section(
                                    (god.0.experience).to_string(),
                                    TextStyle {
                                        font: asset_server.load("fonts/monogram-extended.ttf"),
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
