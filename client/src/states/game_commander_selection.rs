use super::startup::GodAssets;
use crate::{
    cleanup_system,
    components::{
        animation::{AnimationRepeatType, AnimationTimer, TransformAnimation},
        hover::{BoundingBox, ClickEvent, Clickable, Hoverable},
    },
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    prefabs::animation,
    AppState, Cleanup,
};
use bevy::prelude::*;
use protocol::{
    protocol::Protocol,
    protocol_types::heros::{self, God},
};
use reqwest::Method;

const STATE: AppState = AppState::GameCommanderSelection;

pub(crate) struct GameCommanderSelectionPlugin;

impl Plugin for GameCommanderSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(STATE)))
            .add_systems((god_click, on_network).in_set(OnUpdate(STATE)))
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

#[derive(Resource)]
pub(crate) struct GameCommanderSelection(pub Vec<God>);

fn setup(
    mut commands: Commands,
    god_assets: Res<GodAssets>,
    res_gods: Res<GameCommanderSelection>,
) {
    let mut frame_animation = animation::simple(0, 0);
    animation::add_hover_state(&mut frame_animation, 0, 17);

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_translation(Vec3::new(-64.0 * 4.0, 0.0, 5.0)),
                ..Default::default()
            },
            Cleanup,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_assets.god_frame.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(4.0))
                            .with_translation(Vec3::new(0.0, 33.0 * 4.0, 1.0)),
                        ..Default::default()
                    },
                    Hoverable("hover".to_string(), "leave".to_string()),
                    BoundingBox(
                        Vec3::new(48.0, 48.0, 0.0),
                        Quat::from_rotation_z(45.0f32.to_radians()),
                    ),
                    frame_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    GodComponent(res_gods.0[0].clone()),
                    Clickable,
                ))
                .with_children(|parent| {
                    parent.spawn(SpriteSheetBundle {
                        texture_atlas: god_assets
                            .god_portraits
                            .get(&res_gods.0[0].id)
                            .unwrap()
                            .clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(0.25))
                            .with_translation(Vec3::new(0.0, 0.0, -1.0)),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_assets.god_frame.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(4.0))
                            .with_rotation(Quat::from_rotation_z(-90.0f32.to_radians()))
                            .with_translation(Vec3::new(33.0 * 4.0, 0.0, 1.0)),
                        ..Default::default()
                    },
                    Hoverable("hover".to_string(), "leave".to_string()),
                    BoundingBox(
                        Vec3::new(48.0, 48.0, 0.0),
                        Quat::from_rotation_z(45.0f32.to_radians()),
                    ),
                    frame_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    GodComponent(res_gods.0[1].clone()),
                    Clickable,
                ))
                .with_children(|parent| {
                    parent.spawn(SpriteSheetBundle {
                        texture_atlas: god_assets
                            .god_portraits
                            .get(&res_gods.0[1].id)
                            .unwrap()
                            .clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_rotation(Quat::from_rotation_z(
                            90.0f32.to_radians(),
                        ))
                        .with_scale(Vec3::splat(0.25))
                        .with_translation(Vec3::new(0.0, 0.0, -1.0)),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_assets.god_frame.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(4.0))
                            .with_rotation(Quat::from_rotation_z(90.0f32.to_radians()))
                            .with_translation(Vec3::new(-33.0 * 4.0, 0.0, 1.0)),
                        ..Default::default()
                    },
                    Hoverable("hover".to_string(), "leave".to_string()),
                    BoundingBox(
                        Vec3::new(48.0, 48.0, 0.0),
                        Quat::from_rotation_z(45.0f32.to_radians()),
                    ),
                    frame_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    GodComponent(res_gods.0[2].clone()),
                    Clickable,
                ))
                .with_children(|parent| {
                    parent.spawn(SpriteSheetBundle {
                        texture_atlas: god_assets
                            .god_portraits
                            .get(&res_gods.0[2].id)
                            .unwrap()
                            .clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_rotation(Quat::from_rotation_z(
                            -90.0f32.to_radians(),
                        ))
                        .with_scale(Vec3::splat(0.25))
                        .with_translation(Vec3::new(0.0, 0.0, -1.0)),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_assets.god_frame.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(4.0))
                            .with_rotation(Quat::from_rotation_z(180.0f32.to_radians()))
                            .with_translation(Vec3::new(0.0, -33.0 * 4.0, 1.0)),
                        ..Default::default()
                    },
                    Hoverable("hover".to_string(), "leave".to_string()),
                    BoundingBox(
                        Vec3::new(48.0, 48.0, 0.0),
                        Quat::from_rotation_z(45.0f32.to_radians()),
                    ),
                    frame_animation,
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    GodComponent(res_gods.0[3].clone()),
                    Clickable,
                ))
                .with_children(|parent| {
                    parent.spawn(SpriteSheetBundle {
                        texture_atlas: god_assets
                            .god_portraits
                            .get(&res_gods.0[3].id)
                            .unwrap()
                            .clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_rotation(Quat::from_rotation_z(
                            180.0f32.to_radians(),
                        ))
                        .with_scale(Vec3::splat(0.25))
                        .with_translation(Vec3::new(0.0, 0.0, -1.0)),
                        ..Default::default()
                    });
                });
        });
}

#[derive(Component, Debug)]
pub struct GodComponent(pub heros::God);

fn god_click(
    mut ev_clicked: EventReader<ClickEvent>,
    q_god: Query<(&GodComponent, &Transform), With<Clickable>>,
    mut network: ResMut<NetworkingRessource>,
) {
    for ev in ev_clicked.iter() {
        if let Ok((god, _)) = q_god.get(ev.0) {
            network.request(Method::PUT, format!("games/avatar/{}", god.0.id).as_str());
        }
    }
}

fn on_network(
    mut commands: Commands,
    mut ev_network: EventReader<NetworkingEvent>,
    q_god: Query<(Entity, &GodComponent, &Transform)>,
) {
    for ev in ev_network.iter() {
        if let Protocol::AvatarSelectResponse(god) = &ev.0 {
            for (entity, god_comp, transform) in q_god.iter() {
                if god_comp.0.id == god.id {
                    // Move god to center
                    commands.entity(entity).insert(TransformAnimation {
                        source: *transform,
                        target: transform.with_translation(Vec3::ZERO),
                        speed: 1.0,
                        repeat: AnimationRepeatType::Once,
                    });
                } else {
                    // Remove other gods
                    commands.entity(entity).despawn_recursive();
                }
                commands
                    .entity(entity)
                    .remove::<Clickable>()
                    .remove::<Hoverable>();
            }
        }
    }
}
