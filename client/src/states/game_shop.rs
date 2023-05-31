use crate::{
    cleanup_system,
    components::{
        anchors::{AnchorType, Anchors},
        animation::{
            Animation, AnimationDirection, AnimationIndices, AnimationRepeatType, AnimationState,
            AnimationTimer, AnimationTransition, AnimationTransitionType,
        },
        dragndrop::{Dragable, DropEvent, DropTagret},
        hover::{BoundingBox, ClickEvent, Clickable, Hoverable},
    },
    modules::{character::Character, game_user_info::GameUserRes, god::God},
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    prefabs::animation,
    AppState, Cleanup,
};
use bevy::prelude::*;
use protocol::protocol::{BuyRequest, CharacterInstance, GameOpponentInfo, Protocol};
use reqwest::Method;

use super::startup::{CharacterAssets, UiAssets};

const STATE: AppState = AppState::GameShop;

pub(crate) struct GameShopPlugin;

impl Plugin for GameShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShopChangedEvent>()
            .add_event::<BoardChangedEvent>()
            .add_event::<GameUsersChangedEvent>()
            .add_system(setup.in_schedule(OnEnter(STATE)))
            .add_systems(
                (
                    on_network,
                    generate_shop,
                    generate_board,
                    generate_game_users,
                    on_buy,
                    on_move,
                    on_reroll,
                    on_lock,
                    on_sell,
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

#[derive(Component, Debug)]
pub struct Sell;

#[derive(Component, Debug)]
pub struct GameUsers;

#[derive(Debug)]
pub struct ShopChangedEvent(pub Vec<Option<CharacterInstance>>);

#[derive(Debug)]
pub struct BoardChangedEvent(pub Vec<Option<CharacterInstance>>);

#[derive(Debug)]
pub struct GameUsersChangedEvent(pub Vec<GameOpponentInfo>);

fn setup(
    mut commands: Commands,
    mut networking: ResMut<NetworkingRessource>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    res_anchor: Res<Anchors>,
) {
    // root node
    networking.request(Method::GET, "games/shops");
    networking.request(Method::GET, "games/characters");
    networking.request(Method::GET, "games/users/me");
    networking.request(Method::GET, "games/users");
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

            // Bench
            for i in 0..5 {
                parent.spawn((
                    SpriteSheetBundle {
                        texture_atlas: pedestal_atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(2.0)).with_translation(
                            Vec3::new(-34.0 + 68.0 * 2.0 * i as f32, -136.0 * 2.0, 1.0),
                        ),
                        ..Default::default()
                    },
                    Hoverable("hover".to_string(), "leave".to_string()),
                    BoundingBox(Vec3::new(64.0, 64.0, 0.0), Quat::from_rotation_z(0.0)),
                    pedestal_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    DropTagret,
                    Pedestal(7 + i),
                ));
            }
        });

    // Reroll Button
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

    // Lock Button
    let lock = asset_server.load("textures/ui/lock.png");
    let lock_atlas = TextureAtlas::from_grid(lock, Vec2::new(32.0, 32.0), 2, 1, None, None);
    let lock_atlas_handle = texture_atlases.add(lock_atlas);

    let lock_animation = animation::simple(0, 0)
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

    // Sell Area
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("textures/ui/sell.png"),
            transform: Transform::from_translation(Vec3::new(64.0 * 7.0, 200.0, 0.0))
                .with_scale(Vec3::splat(2.0)),
            ..Default::default()
        },
        Hoverable("hover".to_string(), "leave".to_string()),
        BoundingBox(Vec3::new(32.0, 32.0, 0.0), Quat::from_rotation_z(0.0)),
        DropTagret,
        Sell,
        Cleanup,
    ));

    // Opponent Anchor
    commands
        .entity(res_anchor.get(AnchorType::LEFT).unwrap())
        .with_children(|parent| {
            parent.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(80.0, 256.0, 0.0)),
                    ..Default::default()
                },
                GameUsers,
                Cleanup,
            ));
        });
}

fn on_network(
    mut commands: Commands,
    mut networking: ResMut<NetworkingRessource>,
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_shop_change: EventWriter<ShopChangedEvent>,
    mut ev_board_change: EventWriter<BoardChangedEvent>,
    mut ev_game_users_change: EventWriter<GameUsersChangedEvent>,
    mut q_lock: Query<(Entity, &Animation), With<Lock>>,
) {
    for ev in ev_networking.iter() {
        match &ev.0 {
            Protocol::GameShopResponse(user_info, locked, shop) => {
                debug!("GameShopResponse: {:?}", shop);
                commands.insert_resource(GameUserRes(user_info.clone()));
                ev_shop_change.send(ShopChangedEvent(shop.clone()));
                for (entity, animation) in q_lock.iter_mut() {
                    if *locked {
                        if let Some(lock_transition) = animation.get_transition("lock") {
                            commands.entity(entity).insert(lock_transition);
                        }
                    } else if let Some(unlock_transition) = animation.get_transition("unlock") {
                        commands.entity(entity).insert(unlock_transition);
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
            Protocol::SellResponse(user, board) => {
                debug!("SellResponse: {:?}", ev);
                commands.insert_resource(GameUserRes(user.clone()));
                ev_board_change.send(BoardChangedEvent(board.clone()));
            }
            Protocol::GameUsersResponse(users) => {
                debug!("GameUsersResponse: {:?}", ev);
                ev_game_users_change.send(GameUsersChangedEvent(users.clone()));
            }
            Protocol::NetworkingError(err) => {
                if let Some(reference) = err.reference.clone() {
                    match *reference {
                        Protocol::BuyRequest(_) => {
                            debug!("BuyRequest failed: {:?}", err);
                            networking.request(Method::GET, "games/shops");
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
                    Method::POST,
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
        if q_reroll.get(ev.0).is_ok() {
            networking.request(Method::POST, "games/shops");
        }
    }
}

fn on_lock(
    mut ev_cklicked: EventReader<ClickEvent>,
    q_lock: Query<&Lock>,
    mut networking: ResMut<NetworkingRessource>,
) {
    for ev in ev_cklicked.iter() {
        if q_lock.get(ev.0).is_ok() {
            networking.request(Method::PATCH, "games/shops");
        }
    }
}

fn on_sell(
    mut ev_droped: EventReader<DropEvent>,
    q_sell: Query<Entity, With<Sell>>,
    q_character: Query<&BoardCharacter>,
    mut networking: ResMut<NetworkingRessource>,
) {
    for ev in ev_droped.iter() {
        if let Ok(character) = q_sell.get(ev.target).and(q_character.get(ev.entity)) {
            debug!("on_sell: {:?}", character);
            networking.request(
                Method::DELETE,
                format!("games/characters/{}", character.0).as_str(),
            );
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
                    Method::PUT,
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
    character_assets: Res<CharacterAssets>,
    ui_assets: Res<UiAssets>,
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
                            Character(character.clone()),
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn(SpriteBundle {
                                    texture: character_assets.price_orb.clone(),
                                    transform: Transform::from_translation(Vec3::new(
                                        24.0, 28.0, 7.0,
                                    ))
                                    .with_scale(Vec3::splat(0.75)),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(Text2dBundle {
                                        text: Text::from_section(
                                            character.cost.to_string(),
                                            TextStyle {
                                                font: ui_assets.font.clone(),
                                                font_size: 28.0,
                                                color: Color::BLACK,
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

fn generate_game_users(
    mut commands: Commands,
    mut ev_game_users: EventReader<GameUsersChangedEvent>,
    q_game_users: Query<(Option<&Children>, Entity), With<GameUsers>>,
) {
    for ev in ev_game_users.iter() {
        debug!("generate_game_users: {:?}", ev);
        for (children, entity) in q_game_users.iter() {
            if let Some(children) = children {
                for child in children.iter() {
                    commands.entity(*child).despawn_recursive();
                }
            }
            commands.entity(entity).with_children(|parent| {
                for (idx, opponent) in ev.0.iter().enumerate() {
                    parent.spawn((
                        SpatialBundle {
                            transform: Transform::from_translation(Vec3::new(
                                if idx % 2 == 0 { 0.0 } else { 68.0 },
                                -68.0 * idx as f32,
                                1.0,
                            ))
                            .with_scale(Vec3::splat(2.0)),
                            ..Default::default()
                        },
                        God(opponent.clone()),
                    ));
                }
            });
        }
    }
}
