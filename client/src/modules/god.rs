use crate::{
    components::{hover::HoverEvent, tooltip::SetTooltipEvent, ChangeDetectionSystemSet},
    states::{
        game_commander_selection::GodComponent,
        startup::{CharacterAssets, GodAssets, UiAssets},
    },
    util::text::break_text,
    Cleanup,
};
use bevy::prelude::*;
use protocol::{gods::get_gods, protocol::GameOpponentInfo};

pub(crate) struct GodPlugin;

impl Plugin for GodPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_spawn)
            .add_system(on_god_hover.in_set(ChangeDetectionSystemSet::Tooltip));
    }
}

const PORTRAIT_HEIGHT: f32 = 128.0;
const TOOLTIP_NAME_HEIGHT: f32 = 28.0;
const TOOLTIP_NAME_WIDTH: f32 = 150.0;
const TOOLTIP_DESCRIPTION_WIDTH: f32 = 250.0;
const TOOLTIP_DESCRIPTION_HEIGHT: f32 = 200.0;
const TOOLTIP_COLOR: Color = Color::rgba(0.75, 0.75, 0.75, 0.75);

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
            spawn_god_portrait(
                parent,
                &god_assets,
                &ui_assets,
                &character_assets,
                god.0.character_id,
                Some(&god.0),
                false,
            )
        });
    }
}

fn spawn_god_portrait(
    parent: &mut ChildBuilder,
    god_assets: &Res<GodAssets>,
    ui_assets: &Res<UiAssets>,
    character_assets: &Res<CharacterAssets>,
    character_id: i32,
    god: Option<&GameOpponentInfo>,
    full_info: bool,
) {
    let is_next_opponent = god.map(|g| g.is_next_opponent).unwrap_or(false);
    let god_template = &get_gods()[character_id as usize];

    parent
        .spawn((SpriteSheetBundle {
            texture_atlas: god_assets.god_frame.clone(),
            sprite: TextureAtlasSprite::new(if is_next_opponent { 17 } else { 0 }),
            transform: Transform::from_scale(Vec3::splat(1.0))
                .with_translation(Vec3::new(0.0, 0.0, 5.0)),
            ..Default::default()
        },))
        .with_children(|parent| {
            // God Portrait
            parent.spawn(SpriteSheetBundle {
                texture_atlas: god_assets.god_portraits.get(&character_id).unwrap().clone(),
                sprite: TextureAtlasSprite::new(0),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0))
                    .with_scale(Vec3::splat(0.25)),
                ..Default::default()
            });

            if let Some(opponent) = god {
                // Health
                parent
                    .spawn(SpriteBundle {
                        texture: character_assets.health_orb.clone(),
                        transform: Transform::from_translation(Vec3::new(18.0, -28.0, 0.0))
                            .with_scale(Vec3::splat(0.5)),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(Text2dBundle {
                            text: Text::from_section(
                                (opponent.health).to_string(),
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
                // Level
                parent
                    .spawn(SpriteBundle {
                        texture: god_assets.lvl_orb.clone(),
                        transform: Transform::from_translation(Vec3::new(-18.0, -28.0, 0.0))
                            .with_scale(Vec3::splat(0.5)),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(Text2dBundle {
                            text: Text::from_section(
                                (opponent.get_lvl()).to_string(),
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
            }

            if full_info {
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
                                        god_template.name.clone(),
                                        TextStyle {
                                            font: ui_assets.font.clone(),
                                            font_size: 36.0,
                                            color: Color::WHITE,
                                        },
                                    ),
                                    transform: Transform::from_translation(Vec3::new(
                                        0.0, 0.0, 1.0,
                                    )),
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
                                            god_template.description.clone(),
                                            TOOLTIP_DESCRIPTION_WIDTH,
                                            24.0,
                                            true,
                                        ),
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
                    });
            }
        });
}

fn on_god_hover(
    mut commands: Commands,
    mut q_opponent: Query<&God>,
    mut q_god: Query<&GodComponent>,
    mut ev_hover: EventReader<HoverEvent>,
    mut ev_tooltip: EventWriter<SetTooltipEvent>,
    god_assets: Res<GodAssets>,
    ui_assets: Res<UiAssets>,
    character_assets: Res<CharacterAssets>,
) {
    for HoverEvent(entity, is_hovered) in ev_hover.iter() {
        if let Ok(god) = q_opponent.get_mut(*entity).map(|god| &god.0) {
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
                        spawn_god_portrait(
                            parent,
                            &god_assets,
                            &ui_assets,
                            &character_assets,
                            god.character_id,
                            Some(god),
                            true,
                        );
                    })
                    .id();
                ev_tooltip.send(SetTooltipEvent(*entity, Some(tooltip)));
            } else {
                ev_tooltip.send(SetTooltipEvent(*entity, None));
            }
        }

        if let Ok(god) = q_god.get_mut(*entity).map(|god| &god.0) {
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
                        spawn_god_portrait(
                            parent,
                            &god_assets,
                            &ui_assets,
                            &character_assets,
                            god.id,
                            None,
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
