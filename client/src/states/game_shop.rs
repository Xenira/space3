use std::time::Duration;

use bevy::{prelude::*, utils::tracing::span::Entered};
use protocol::{protocol::Protocol, protocol_types::character::Character};
use surf::http::Method;

use crate::{
    cleanup_system,
    components::{
        animation::AnimationTimer,
        dragndrop::{Dragable, DropTagret},
        hover::{BoundingBox, Clickable, Hoverable},
    },
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    prefabs::animation,
    states::game_commander_selection::GodComponent,
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::GameShop;

pub(crate) struct GameShopPlugin;

impl Plugin for GameShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShopChangedEvent>()
            .add_system(setup.in_schedule(OnEnter(STATE)))
            .add_systems((on_network, generate_shop).in_set(OnUpdate(STATE)))
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

#[derive(Component)]
pub struct Shop;

#[derive(Component)]
pub struct ShopCharacter(pub Character);

#[derive(Component)]
pub struct Pedestals;

#[derive(Component)]
pub struct Pedestal(pub u8);

#[derive(Debug)]
pub struct ShopChangedEvent(pub Vec<Character>);

fn setup(
    mut commands: Commands,
    mut networking: ResMut<NetworkingRessource>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    // root node
    networking.request(Method::Get, "games/shop");
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(-64.0 * 4.0, 200.0, 0.0)),
            ..Default::default()
        },
        Shop,
        Cleanup,
    ));

    // spawn character pedestals
    let pedestal = asset_server.load("textures/ui/character_base.png");
    let pedestal_atlas = TextureAtlas::from_grid(pedestal, Vec2::new(64.0, 64.0), 2, 1, None, None);
    let pedestal_atlas_handle = texture_atlases.add(pedestal_atlas);

    let mut pedestal_animation = animation::simple(0, 0);
    animation::add_hover_state(&mut pedestal_animation, 1, 1);

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_translation(Vec3::new(-64.0 * 4.0, 0.0, 0.0)),
                ..Default::default()
            },
            Pedestals,
            Cleanup,
        ))
        .with_children(|parent| {
            // front row
            for i in 0..4 {
                parent.spawn((
                    SpriteSheetBundle {
                        texture_atlas: pedestal_atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(2.0))
                            .with_translation(Vec3::new(68.0 * 2.0 * i as f32, 0.0, 1.0)),
                        ..Default::default()
                    },
                    Hoverable("hover".to_string(), "leave".to_string()),
                    BoundingBox(Vec3::new(64.0, 64.0, 0.0), Quat::from_rotation_z(0.0)),
                    pedestal_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    DropTagret,
                    Pedestal(i),
                ));
            }
            // back row
            for i in 0..3 {
                parent.spawn((
                    SpriteSheetBundle {
                        texture_atlas: pedestal_atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(2.0))
                            .with_translation(Vec3::new(68.0 + 68.0 * 2.0 * i as f32, -136.0, 1.0)),
                        ..Default::default()
                    },
                    Hoverable("hover".to_string(), "leave".to_string()),
                    BoundingBox(Vec3::new(64.0, 64.0, 0.0), Quat::from_rotation_z(0.0)),
                    pedestal_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    DropTagret,
                    Pedestal(i),
                ));
            }
        });
}

fn on_network(
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_shop_change: EventWriter<ShopChangedEvent>,
) {
    for ev in ev_networking.iter() {
        match &ev.0 {
            Protocol::GameShopResponse(shop) => {
                debug!("GameShopResponse: {:?}", shop);
                ev_shop_change.send(ShopChangedEvent(shop.clone()));
            }
            _ => {}
        }
    }
}

fn generate_shop(
    mut commands: Commands,
    mut ev_shop_change: EventReader<ShopChangedEvent>,
    q_shop: Query<Entity, With<Shop>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_shop_change.iter() {
        let shop_frame = asset_server.load("textures/ui/user_frame.png");
        let shop_frame_atlas =
            TextureAtlas::from_grid(shop_frame, Vec2::new(64.0, 64.0), 2, 1, None, None);
        let shop_frame_atlas_handle = texture_atlases.add(shop_frame_atlas);

        let mut frame_animation = animation::simple(0, 0);
        animation::add_hover_state(&mut frame_animation, 1, 1);

        let character_fallback = asset_server.load("textures/ui/character_fallback.png");

        debug!("generate_shop: {:?}", ev);
        let shop = q_shop.single();
        commands.entity(shop).despawn_descendants();

        commands.entity(shop).with_children(|parent| {
            for (i, character) in ev.0.iter().enumerate() {
                parent
                    .spawn((
                        SpriteSheetBundle {
                            texture_atlas: shop_frame_atlas_handle.clone(),
                            sprite: TextureAtlasSprite::new(0),
                            transform: Transform::from_scale(Vec3::splat(2.0))
                                .with_translation(Vec3::new(68.0 * 2.0 * i as f32, 0.0, 1.0)),
                            ..Default::default()
                        },
                        Hoverable("hover".to_string(), "leave".to_string()),
                        BoundingBox(Vec3::new(64.0, 64.0, 0.0), Quat::from_rotation_z(0.0)),
                        frame_animation.clone(),
                        AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                        Clickable,
                        ShopCharacter(character.clone()),
                        Dragable,
                    ))
                    .with_children(|parent| {
                        parent.spawn(SpriteBundle {
                            texture: character_fallback.clone(),
                            ..Default::default()
                        });
                    });
            }
        });
    }
}
