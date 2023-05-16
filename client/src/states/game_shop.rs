use std::time::Duration;

use bevy::{prelude::*, utils::tracing::span::Entered};
use protocol::{
    protocol::{BuyRequest, CharacterInstance, Protocol},
    protocol_types::character::Character,
};
use surf::http::Method;

use crate::{
    cleanup_system,
    components::{
        animation::AnimationTimer,
        dragndrop::{Dragable, DropEvent, DropTagret},
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
            .add_event::<BoardChangedEvent>()
            .add_system(setup.in_schedule(OnEnter(STATE)))
            .add_systems(
                (on_network, generate_shop, on_buy, on_move, generate_board)
                    .in_set(OnUpdate(STATE)),
            )
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

#[derive(Component)]
pub struct Shop;

#[derive(Component, Debug)]
pub struct ShopCharacter {
    idx: u8,
    character: Character,
}

#[derive(Component)]
pub struct Pedestals;

#[derive(Component, Debug)]
pub struct Pedestal(pub u8);

#[derive(Component, Debug)]
pub struct BoardCharacter(pub u8, pub CharacterInstance);

#[derive(Debug)]
pub struct ShopChangedEvent(pub Vec<Option<Character>>);

#[derive(Debug)]
pub struct BoardChangedEvent(pub Vec<Option<CharacterInstance>>);

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
    animation::add_hover_state(&mut pedestal_animation, 0, 1);

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
                    Pedestal(4 + i),
                ));
            }
        });
}

fn on_network(
    mut networking: ResMut<NetworkingRessource>,
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_shop_change: EventWriter<ShopChangedEvent>,
    mut ev_board_change: EventWriter<BoardChangedEvent>,
) {
    for ev in ev_networking.iter() {
        match &ev.0 {
            Protocol::GameShopResponse(shop) => {
                debug!("GameShopResponse: {:?}", shop);
                ev_shop_change.send(ShopChangedEvent(shop.clone()));
            }
            Protocol::BuyResponse(shop, board) => {
                debug!("BuyResponse: {:?}", ev);
                ev_shop_change.send(ShopChangedEvent(shop.clone()));
                ev_board_change.send(BoardChangedEvent(board.clone()));
            }
            Protocol::BoardResponse(board) => {
                debug!("MoveResponse: {:?}", ev);
                ev_board_change.send(BoardChangedEvent(board.clone()));
            }
            Protocol::NetworkingError(err) => {
                if let Some(reference) = err.reference.clone() {
                    match *reference {
                        Protocol::BuyRequest(_) => {
                            debug!("BuyRequest failed: {:?}", err);
                            networking.request(Method::Get, "games/shop");
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn on_buy(
    mut commands: Commands,
    mut ev_droped: EventReader<DropEvent>,
    q_pedestal: Query<&Pedestal>,
    q_god: Query<&ShopCharacter>,
    mut networking: ResMut<NetworkingRessource>,
) {
    for ev in ev_droped.iter() {
        if let Ok(pedestal) = q_pedestal.get(ev.target) {
            if let Ok(god) = q_god.get(ev.entity) {
                debug!("on_buy: {:?} {:?}", pedestal, god);
                networking.request_data(
                    Method::Post,
                    "games/shop/buy",
                    &BuyRequest {
                        character_idx: god.idx,
                        target_idx: pedestal.0,
                    },
                );
                commands.entity(ev.entity).despawn_recursive();
            }
        }
        debug!("on_buy: {:?}", ev);
    }
}

fn on_move(
    mut commands: Commands,
    mut ev_droped: EventReader<DropEvent>,
    q_pedestal: Query<&Pedestal>,
    q_god: Query<&BoardCharacter>,
    mut networking: ResMut<NetworkingRessource>,
) {
    for ev in ev_droped.iter() {
        if let Ok(pedestal) = q_pedestal.get(ev.target) {
            if let Ok(god) = q_god.get(ev.entity) {
                debug!("on_move: {:?} {:?}", pedestal, god);
                networking.request(
                    Method::Put,
                    format!("games/character/{}/{}", god.0, pedestal.0).as_str(),
                );
                commands.entity(ev.entity).despawn_recursive();
            }
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
        animation::add_hover_state(&mut frame_animation, 0, 1);

        let character_fallback = asset_server.load("textures/ui/character_fallback.png");

        debug!("generate_shop: {:?}", ev);
        let shop = q_shop.single();
        commands.entity(shop).despawn_descendants();

        commands.entity(shop).with_children(|parent| {
            for (i, character) in ev.0.iter().enumerate() {
                if let Some(character) = character {
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
                            ShopCharacter {
                                idx: i as u8,
                                character: character.clone(),
                            },
                            Dragable,
                        ))
                        .with_children(|parent| {
                            parent.spawn(SpriteBundle {
                                texture: character_fallback.clone(),
                                ..Default::default()
                            });
                        });
                }
            }
        });
    }
}

fn generate_board(
    mut commands: Commands,
    mut ev_shop_change: EventReader<BoardChangedEvent>,
    q_board_character: Query<(Entity, &BoardCharacter)>,
    q_pedestal: Query<(Entity, &Pedestal)>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_shop_change.iter() {
        let shop_frame = asset_server.load("textures/ui/user_frame.png");
        let shop_frame_atlas =
            TextureAtlas::from_grid(shop_frame, Vec2::new(64.0, 64.0), 2, 1, None, None);
        let shop_frame_atlas_handle = texture_atlases.add(shop_frame_atlas);

        let mut frame_animation = animation::simple(0, 0);
        animation::add_hover_state(&mut frame_animation, 0, 1);

        let character_fallback = asset_server.load("textures/ui/character_fallback.png");

        for (entity, _) in q_board_character.iter() {
            commands.entity(entity).despawn_recursive();
        }

        for (idx, pedestal) in
            ev.0.iter()
                .enumerate()
                .filter(|(_, c)| c.is_some())
                .map(|(idx, _)| {
                    (
                        idx,
                        q_pedestal
                            .iter()
                            .find(|(_, pedestal)| pedestal.0 == idx as u8),
                    )
                })
                .filter(|(_, pedestal)| pedestal.is_some())
        {
            if let Some((entity, _)) = pedestal {
                commands.entity(entity).with_children(|parent| {
                    parent
                        .spawn((
                            SpriteSheetBundle {
                                texture_atlas: shop_frame_atlas_handle.clone(),
                                sprite: TextureAtlasSprite::new(0),
                                ..Default::default()
                            },
                            Hoverable("hover".to_string(), "leave".to_string()),
                            BoundingBox(Vec3::new(64.0, 64.0, 0.0), Quat::from_rotation_z(0.0)),
                            frame_animation.clone(),
                            AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                            Clickable,
                            BoardCharacter(idx as u8, ev.0[idx].clone().unwrap()),
                            Dragable,
                        ))
                        .with_children(|parent| {
                            parent.spawn(SpriteBundle {
                                texture: character_fallback.clone(),
                                ..Default::default()
                            });
                        });
                });
            }
        }
    }
}
