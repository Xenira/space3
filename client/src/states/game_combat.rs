use crate::{
    cleanup_system,
    components::{
        anchors::{AnchorType, Anchors},
        animation::{Animation, AnimationFinished, AnimationRepeatType, TransformAnimation},
    },
    modules::{character::Character, god::God},
    AppState, Cleanup,
};
use bevy::prelude::*;
use protocol::protocol::{BattleResponse, CharacterInstance, GameOpponentInfo};

use super::game_shop::BoardCharacter;

const STATE: AppState = AppState::GameBattle;

pub(crate) struct GameCombatPlugin;

impl Plugin for GameCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameCombatState>()
            .add_event::<BattleBoardChangedEvent>()
            .add_system(setup.in_schedule(OnEnter(STATE)))
            .add_system((generate_board).in_set(OnUpdate(STATE)))
            .add_system(
                play_animation
                    .in_schedule(OnEnter(GameCombatState::AnimationFinished))
                    .run_if(in_state(STATE)),
            )
            .add_system(
                animation_finished
                    .in_set(OnUpdate(GameCombatState::PlayAnimation))
                    .run_if(in_state(STATE)),
            )
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum GameCombatState {
    #[default]
    Setup,
    PlayAnimation,
    AnimationFinished,
    WaitingForShop,
}

#[derive(Component, Debug)]
pub struct BoardOwn;

#[derive(Component, Debug)]
pub struct BoardOpponent;

#[derive(Component, Debug)]
pub struct OpponentProfile;

#[derive(Resource, Debug)]
pub struct BattleRes(pub BattleResponse);

#[derive(Debug)]
pub struct BattleBoardChangedEvent(pub [Vec<Option<CharacterInstance>>; 2]);

fn setup(
    mut commands: Commands,
    state: Res<BattleRes>,
    mut ev_board_change: EventWriter<BattleBoardChangedEvent>,
    res_anchor: Res<Anchors>,
) {
    info!("Setting up game combat state");

    commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(-64.0 * 4.0, -128.0, 0.0)),
            ..Default::default()
        },
        BoardOwn,
        Cleanup,
    ));
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(-64.0 * 4.0, 128.0, 0.0)),
            ..Default::default()
        },
        BoardOpponent,
        Cleanup,
    ));

    // Spawn enemy profile
    commands
        .entity(res_anchor.get(AnchorType::TOP_RIGHT).unwrap())
        .with_children(|parent| {
            parent.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(-128.0, -128.0, 10.0))
                        .with_scale(Vec3::splat(3.0)),
                    ..Default::default()
                },
                God(state.0.opponent.clone()),
                OpponentProfile,
                Cleanup,
            ));
        });

    ev_board_change.send(BattleBoardChangedEvent([
        state.0.start_own.clone(),
        state.0.start_opponent.clone(),
    ]));
}

fn play_animation(
    mut commands: Commands,
    mut state: ResMut<BattleRes>,
    mut combat_state: ResMut<NextState<GameCombatState>>,
    q_board_character: Query<(
        Entity,
        &BoardCharacter,
        &Children,
        &GlobalTransform,
        &Transform,
    )>,
    q_animation: Query<(Entity, &Animation)>,
    q_target: Query<(&GlobalTransform, &BoardCharacter)>,
    mut ev_board_change: EventWriter<BattleBoardChangedEvent>,
) {
    let current_action = state.0.actions.first().cloned();
    if let Some(current_action) = current_action {
        if let Some((entity, character, children, source_global_transform, source_transform)) =
            q_board_character
                .iter()
                .find(|(_, board_character, _, _, _)| board_character.1.id == current_action.source)
        {
            match current_action.action {
                protocol::protocol::BattleActionType::Attack => {
                    if let Some(target) = current_action.target {
                        if let Some((transform, _)) = q_target
                            .iter()
                            .find(|(_, board_character)| board_character.1.id == target)
                        {
                            debug!("Playing animation for {:?}", character);
                            let target_transform = transform.compute_transform().translation;
                            let target_transform = target_transform
                                - (source_global_transform.compute_transform().translation
                                    - source_transform.translation);
                            commands.entity(entity).insert(TransformAnimation {
                                source: source_transform.clone(),
                                target: Transform::from_translation(target_transform)
                                    .with_scale(source_transform.scale),
                                speed: 5.0,
                                repeat: AnimationRepeatType::PingPongOnce,
                            });
                        } else {
                            warn!("No target found for {:?}", current_action);
                        }
                    } else {
                        warn!("No target found for {:?}", current_action);
                    }
                }
                protocol::protocol::BattleActionType::Die => {
                    debug!("Playing animation for {:?}", character);
                    if let Some((entity, animation)) = children
                        .iter()
                        .find_map(|entity| q_animation.get(*entity).ok())
                    {
                        commands
                            .entity(entity)
                            .insert(animation.get_transition("die").unwrap());
                    } else {
                        warn!("No animation found for {:?}", character);
                        state.0.actions.remove(0);
                        ev_board_change.send(BattleBoardChangedEvent([
                            current_action.result_own.clone(),
                            current_action.result_opponent.clone(),
                        ]));
                    }
                }
                _ => (),
            }
            debug!("Changing state to PlayAnimation");
            combat_state.set(GameCombatState::PlayAnimation);
        } else {
            warn!("No character found for {:?}", current_action);
            combat_state.set(GameCombatState::AnimationFinished);
        }
    } else {
        combat_state.set(GameCombatState::WaitingForShop);
    }
}

fn animation_finished(
    mut battle: ResMut<BattleRes>,
    q_board_character: Query<(Entity, &Children, &BoardCharacter)>,
    mut ev_animation_finished: EventReader<AnimationFinished>,
    mut ev_board_change: EventWriter<BattleBoardChangedEvent>,
) {
    if let Some(current_action) = battle.0.actions.first().cloned() {
        for ev in ev_animation_finished.iter() {
            debug!("Animation finished for {:?}", ev.0);
            if q_board_character
                .iter()
                .any(|(entity, children, board_character)| {
                    (ev.0 == entity || children.contains(&ev.0))
                        && (board_character.1.id == current_action.source
                            || (current_action.target.is_some()
                                && board_character.1.id == current_action.target.unwrap()))
                })
            {
                debug!(
                    "Animation finished for {:?} on entity {:?}",
                    current_action, ev.0
                );
                battle.0.actions.remove(0);
                ev_board_change.send(BattleBoardChangedEvent([
                    current_action.result_own.clone(),
                    current_action.result_opponent.clone(),
                ]));
            }
        }
    }
}

fn generate_board(
    mut commands: Commands,
    mut combat_state: ResMut<NextState<GameCombatState>>,
    mut ev_shop_change: EventReader<BattleBoardChangedEvent>,
    q_board_character: Query<(Entity, &BoardCharacter)>,
    q_own: Query<Entity, With<BoardOwn>>,
    q_opponent: Query<Entity, With<BoardOpponent>>,
) {
    for ev in ev_shop_change.iter() {
        debug!("Generating board");

        for (entity, _) in q_board_character.iter() {
            commands.entity(entity).despawn_recursive();
        }

        for (player_idx, idx, board, character) in
            ev.0.iter()
                .enumerate()
                .map(|(player_idx, player)| {
                    (
                        player_idx,
                        if player_idx == 0 {
                            q_own.get_single()
                        } else {
                            q_opponent.get_single()
                        },
                        player,
                    )
                })
                .filter(|(_, entity, _)| entity.is_ok())
                .map(|(player_idx, entity, player)| (player_idx, entity.unwrap(), player))
                .flat_map(|(player_idx, entity, player)| {
                    player
                        .iter()
                        .enumerate()
                        .map(move |(idx, character)| (player_idx, idx, entity, character.as_ref()))
                })
                .filter(|(_, _, _, character)| character.is_some())
                .map(|(player_idx, idx, entity, character)| {
                    (player_idx, idx, entity, character.unwrap())
                })
        {
            commands.entity(board).with_children(|parent| {
                parent.spawn((
                    SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(
                            68.0 * 2.0 * (idx % 4) as f32 + if idx < 4 { 0.0 } else { 68.0 } as f32,
                            if idx < 4 {
                                0.0
                            } else if player_idx == 0 {
                                -136.0
                            } else {
                                136.0
                            },
                            0.0,
                        ))
                        .with_scale(Vec3::splat(2.0)),
                        ..Default::default()
                    },
                    Character(character.clone()),
                    BoardCharacter(idx as u8, character.clone()),
                ));
            });
        }
        combat_state.set(GameCombatState::AnimationFinished);
    }
}
