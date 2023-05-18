use crate::{
    cleanup_system,
    components::{
        animation::{AnimationIndices, AnimationRepeatType, AnimationTimer, TransformAnimation},
        hover::{BoundingBox, ClickEvent, Clickable, Hoverable, Hovered},
    },
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    prefabs::animation,
    util::text::break_text,
    AppState, Cleanup,
};
use bevy::prelude::*;
use protocol::{
    protocol::Protocol,
    protocol_types::heros::{self, God},
};
use surf::http::Method;

const STATE: AppState = AppState::GameCommanderSelection;

pub(crate) struct GameCommanderSelectionPlugin;

impl Plugin for GameCommanderSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(STATE)))
            .add_systems((god_hover, god_click, on_network).in_set(OnUpdate(STATE)))
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

#[derive(Resource)]
pub(crate) struct GameCommanderSelection(pub Vec<God>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    res_gods: Res<GameCommanderSelection>,
) {
    let god_frame = asset_server.load("textures/ui/god_frame2.png");
    let god_frame_atlas =
        TextureAtlas::from_grid(god_frame, Vec2::new(64.0, 64.0), 18, 1, None, None);
    let god_frame_atlas_handle = texture_atlases.add(god_frame_atlas);

    let mut frame_animation = animation::simple(0, 0);
    animation::add_hover_state(&mut frame_animation, 0, 17);

    let god_fallback = asset_server.load("textures/ui/god_fallback.png");

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_translation(Vec3::new(-64.0 * 4.0, 0.0, 0.0)),
                ..Default::default()
            },
            Cleanup,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_frame_atlas_handle.clone(),
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
                    parent.spawn(SpriteBundle {
                        texture: god_fallback.clone(),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_frame_atlas_handle.clone(),
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
                    parent.spawn(SpriteBundle {
                        texture: god_fallback.clone(),
                        transform: Transform::from_rotation(Quat::from_rotation_z(
                            90.0f32.to_radians(),
                        )),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_frame_atlas_handle.clone(),
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
                    parent.spawn(SpriteBundle {
                        texture: god_fallback.clone(),
                        transform: Transform::from_rotation(Quat::from_rotation_z(
                            -90.0f32.to_radians(),
                        )),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_frame_atlas_handle.clone(),
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
                    parent.spawn(SpriteBundle {
                        texture: god_fallback.clone(),
                        transform: Transform::from_rotation(Quat::from_rotation_z(
                            180.0f32.to_radians(),
                        )),
                        ..Default::default()
                    });
                });
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::End,
                    ..Default::default()
                },
                ..Default::default()
            },
            Cleanup,
        ))
        .with_children(|parent| {
            parent
                .spawn((NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(40.0), Val::Percent(100.0)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        gap: Size::height(Val::Px(24.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Select your deity",
                            TextStyle {
                                font: asset_server.load("fonts/monogram-extended.ttf"),
                                font_size: 50.0,
                                color: Color::WHITE,
                            },
                        ),
                        GodTitle,
                    ));
                    parent.spawn((
                        TextBundle::from_sections([TextSection::from_style(TextStyle {
                            font: asset_server.load("fonts/monogram-extended.ttf"),
                            font_size: 28.0,
                            color: Color::WHITE,
                        })]),
                        GodDescription,
                    ));
                    parent.spawn((
                        TextBundle::from_sections([TextSection::from_style(TextStyle {
                            font: asset_server.load("fonts/monogram-extended.ttf"),
                            font_size: 32.0,
                            color: Color::WHITE,
                        })]),
                        GodPantheon,
                    ));
                });
        });
}

#[derive(Component, Debug)]
pub struct GodTitle;

#[derive(Component, Debug)]
pub struct GodDescription;

#[derive(Component, Debug)]
pub struct GodPantheon;

#[derive(Component, Debug)]
pub struct GodComponent(pub heros::God);

fn god_hover(
    q_hovered: Query<&GodComponent, Added<Hovered>>,
    mut q_title: Query<&mut Text, With<GodTitle>>,
    mut q_desc: Query<&mut Text, (With<GodDescription>, Without<GodTitle>)>,
    mut q_pantheon: Query<
        &mut Text,
        (
            With<GodPantheon>,
            Without<GodTitle>,
            Without<GodDescription>,
        ),
    >,
) {
    if let Some(god) = q_hovered.iter().next() {
        q_title.get_single_mut().unwrap().sections[0].value = god.0.name.to_string();
        q_desc.get_single_mut().unwrap().sections[0].value =
            break_text(god.0.description.to_string(), 36, true);
        q_pantheon.get_single_mut().unwrap().sections[0].value = god.0.pantheon.to_string();
    }
}

fn god_click(
    mut commands: Commands,
    mut ev_clicked: EventReader<ClickEvent>,
    q_god: Query<(&GodComponent, &Transform), With<Clickable>>,
    mut network: ResMut<NetworkingRessource>,
) {
    for ev in ev_clicked.iter() {
        q_god.get(ev.0).ok().map(|(god, transform)| {
            network.request(Method::Put, format!("games/avatar/{}", god.0.id).as_str());
        });
    }
}

fn on_network(
    mut commands: Commands,
    mut ev_network: EventReader<NetworkingEvent>,
    q_god: Query<(Entity, &GodComponent, &Transform)>,
    mut q_title: Query<&mut Text, With<GodTitle>>,
    mut q_desc: Query<&mut Text, (With<GodDescription>, Without<GodTitle>)>,
    mut q_pantheon: Query<
        &mut Text,
        (
            With<GodPantheon>,
            Without<GodTitle>,
            Without<GodDescription>,
        ),
    >,
) {
    for ev in ev_network.iter() {
        if let Protocol::AvatarSelectResponse(god) = &ev.0 {
            for (entity, god_comp, transform) in q_god.iter() {
                if god_comp.0.id == god.id {
                    // Move god to center
                    commands.entity(entity).insert(TransformAnimation {
                        source: transform.clone(),
                        target: transform.with_translation(Vec3::ZERO),
                        speed: 1.0,
                        repeat: AnimationRepeatType::Once,
                    });

                    // Set god info
                    q_title.get_single_mut().unwrap().sections[0].value = god.name.to_string();
                    q_desc.get_single_mut().unwrap().sections[0].value =
                        break_text(god.description.to_string(), 36, true);
                    q_pantheon.get_single_mut().unwrap().sections[0].value =
                        god.pantheon.to_string();
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
