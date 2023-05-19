use bevy::prelude::*;
use protocol::protocol::{BuyRequest, CharacterInstance, GameUserInfo, Protocol};
use surf::http::Method;

use crate::{
    cleanup_system,
    components::{
        animation::{
            Animation, AnimationDirection, AnimationIndices, AnimationRepeatType, AnimationState,
            AnimationTimer, AnimationTransition, AnimationTransitionType,
        },
        dragndrop::{Dragable, DropEvent, DropTagret},
        hover::{BoundingBox, ClickEvent, Clickable, Hoverable},
    },
    modules::{character::Character, game_user_info::GameUserRes},
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    prefabs::animation,
    AppState, Cleanup,
};

const STATE: AppState = AppState::GameShop;

pub(crate) struct GameShopPlugin;

impl Plugin for GameShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShopChangedEvent>()
            .add_event::<BoardChangedEvent>()
            .add_system(setup.in_schedule(OnEnter(STATE)))
            .add_systems(
                (
                    on_network,
                    generate_shop,
                    on_buy,
                    on_move,
                    generate_board,
                    on_reroll,
                    on_lock,
                )
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
    character: CharacterInstance,
}

#[derive(Component)]
pub struct Pedestals;

#[derive(Component, Debug)]
pub struct Pedestal(pub u8);

#[derive(Component, Debug)]
pub struct BoardCharacter(pub u8, pub CharacterInstance);

#[derive(Component, Debug)]
pub struct Reroll;

#[derive(Component, Debug)]
pub struct Lock;

#[derive(Debug)]
pub struct ShopChangedEvent(pub Vec<Option<(u8, CharacterInstance)>>);

#[derive(Debug)]
pub struct BoardChangedEvent(pub Vec<Option<CharacterInstance>>);

fn setup(
    mut commands: Commands,
    mut networking: ResMut<NetworkingRessource>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    // root node
    networking.request(Method::Get, "games/shops");
    networking.request(Method::Get, "games/characters");
    networking.request(Method::Get, "games/users/me");
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

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("textures/ui/reroll.png"),
            transform: Transform::from_translation(Vec3::new(64.0 * 6.0, 200.0, 0.0)),
            ..Default::default()
        },
        Hoverable("hover".to_string(), "leave".to_string()),
        BoundingBox(Vec3::new(32.0, 32.0, 0.0), Quat::from_rotation_z(0.0)),
        Clickable,
        Reroll,
        Cleanup,
    ));

    let lock = asset_server.load("textures/ui/lock.png");
    let lock_atlas = TextureAtlas::from_grid(lock, Vec2::new(32.0, 32.0), 2, 1, None, None);
    let lock_atlas_handle = texture_atlases.add(lock_atlas);

    let mut lock_animation = animation::simple(0, 0)
        .with_state(
            AnimationState::new("lock", AnimationIndices::new(0, 1))
                .with_repeat_type(AnimationRepeatType::Once)
                .with_direction(AnimationDirection::Backward),
        )
        .with_state(
            AnimationState::new("unlock", AnimationIndices::new(0, 1))
                .with_repeat_type(AnimationRepeatType::Once),
        )
        .with_global_transition(AnimationTransition {
            name: "lock".to_string(),
            to_state: "lock".to_string(),
            transition_type: AnimationTransitionType::Imediate,
        })
        .with_global_transition(AnimationTransition {
            name: "unlock".to_string(),
            to_state: "unlock".to_string(),
            transition_type: AnimationTransitionType::Imediate,
        });

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: lock_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_translation(Vec3::new(64.0 * -5.5, 200.0, 0.0)),
            ..Default::default()
        },
        lock_animation,
        AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
        Hoverable("hover".to_string(), "leave".to_string()),
        BoundingBox(Vec3::new(32.0, 32.0, 0.0), Quat::from_rotation_z(0.0)),
        Clickable,
        Lock,
        Cleanup,
    ));
}

fn on_network(
    mut commands: Commands,
    mut networking: ResMut<NetworkingRessource>,
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_shop_change: EventWriter<ShopChangedEvent>,
    mut ev_board_change: EventWriter<BoardChangedEvent>,
    mut q_lock: Query<(Entity, &Animation), With<Lock>>,
) {
    for ev in ev_networking.iter() {
        match &ev.0 {
            Protocol::GameShopResponse(locked, shop) => {
                debug!("GameShopResponse: {:?}", shop);
                ev_shop_change.send(ShopChangedEvent(shop.clone()));
                for (entity, animation) in q_lock.iter_mut() {
                    if *locked {
                        if let Some(lock_transition) = animation.get_transition("lock") {
                            commands.entity(entity).insert(lock_transition);
                        }
                    } else {
                        if let Some(unlock_transition) = animation.get_transition("unlock") {
                            commands.entity(entity).insert(unlock_transition);
                        }
                    }
                }
            }
            Protocol::BuyResponse(user_info, shop, board) => {
                debug!("BuyResponse: {:?}", ev);
                commands.insert_resource(GameUserRes(user_info.clone()));
                ev_shop_change.send(ShopChangedEvent(shop.clone()));
                ev_board_change.send(BoardChangedEvent(board.clone()));
            }
            Protocol::BoardResponse(board) => {
                debug!("BoardResponse: {:?}", ev);
                ev_board_change.send(BoardChangedEvent(board.clone()));
            }
            Protocol::NetworkingError(err) => {
                if let Some(reference) = err.reference.clone() {
                    match *reference {
                        Protocol::BuyRequest(_) => {
                            debug!("BuyRequest failed: {:?}", err);
                            networking.request(Method::Get, "games/shops");
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
                    "games/shops/buy",
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

fn on_reroll(
    mut ev_cklicked: EventReader<ClickEvent>,
    q_reroll: Query<&Reroll>,
    mut networking: ResMut<NetworkingRessource>,
) {
    for ev in ev_cklicked.iter() {
        if let Ok(_) = q_reroll.get(ev.0) {
            networking.request(Method::Post, "games/shops");
        }
    }
}

fn on_lock(
    mut ev_cklicked: EventReader<ClickEvent>,
    q_lock: Query<&Lock>,
    mut networking: ResMut<NetworkingRessource>,
) {
    for ev in ev_cklicked.iter() {
        if let Ok(_) = q_lock.get(ev.0) {
            networking.request(Method::Patch, "games/shops");
        }
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
                    format!("games/characters/{}/{}", god.0, pedestal.0).as_str(),
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

        debug!("generate_shop: {:?}", ev);
        let shop = q_shop.single();
        commands.entity(shop).despawn_descendants();

        commands.entity(shop).with_children(|parent| {
            for (i, character) in ev.0.iter().enumerate() {
                if let Some((cost, character)) = character {
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
                            Character(character.clone()),
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn(SpriteBundle {
                                    texture: asset_server.load("textures/ui/price_orb.png"),
                                    transform: Transform::from_translation(Vec3::new(
                                        24.0, 28.0, 2.0,
                                    ))
                                    .with_scale(Vec3::splat(0.75)),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(Text2dBundle {
                                        text: Text::from_section(
                                            cost.to_string(),
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
                        });
                }
            }
        });
    }
}

fn generate_board(
    mut commands: Commands,
    mut ev_board_change: EventReader<BoardChangedEvent>,
    q_board_character: Query<(Entity, &BoardCharacter)>,
    q_pedestal: Query<(Entity, &Pedestal)>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_board_change.iter() {
        let shop_frame = asset_server.load("textures/ui/user_frame.png");
        let shop_frame_atlas =
            TextureAtlas::from_grid(shop_frame, Vec2::new(64.0, 64.0), 2, 1, None, None);
        let shop_frame_atlas_handle = texture_atlases.add(shop_frame_atlas);

        let mut frame_animation = animation::simple(0, 0);
        animation::add_hover_state(&mut frame_animation, 0, 1);

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
                    parent.spawn((
                        SpriteSheetBundle {
                            texture_atlas: shop_frame_atlas_handle.clone(),
                            sprite: TextureAtlasSprite::new(0),
                            transform: Transform::from_translation(Vec3::ZERO),
                            ..Default::default()
                        },
                        Hoverable("hover".to_string(), "leave".to_string()),
                        BoundingBox(Vec3::new(64.0, 64.0, 1.0), Quat::from_rotation_z(0.0)),
                        frame_animation.clone(),
                        AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                        Clickable,
                        BoardCharacter(idx as u8, ev.0[idx].clone().unwrap()),
                        Dragable,
                        Character(ev.0[idx].clone().unwrap()),
                    ));
                });
            }
        }
    }
}
