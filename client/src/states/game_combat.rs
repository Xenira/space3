use crate::{
    cleanup_system,
    components::animation::{
        Animation, AnimationFinished, AnimationIndices, AnimationRepeatType, AnimationState,
        AnimationTimer, AnimationTransition, AnimationTransitionType, TransformAnimation,
    },
    prefabs::animation,
    AppState, Cleanup, StateChangeEvent,
};
use bevy::{prelude::*, transform::commands};
use protocol::protocol::{BattleResponse, CharacterInstance};

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

#[derive(Resource, Debug)]
pub struct BattleRes(pub BattleResponse);

#[derive(Debug)]
pub struct BattleBoardChangedEvent(pub [Vec<Option<CharacterInstance>>; 2]);

fn setup(
    mut commands: Commands,
    state: Res<BattleRes>,
    mut ev_board_change: EventWriter<BattleBoardChangedEvent>,
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

    ev_board_change.send(BattleBoardChangedEvent([
        state.0.start_own.clone(),
        state.0.start_opponent.clone(),
    ]));
}

fn play_animation(
    mut commands: Commands,
    state: Res<BattleRes>,
    mut combat_state: ResMut<NextState<GameCombatState>>,
    q_board_character: Query<(
        Entity,
        &BoardCharacter,
        &Animation,
        &GlobalTransform,
        &Transform,
    )>,
    q_target: Query<(&GlobalTransform, &BoardCharacter)>,
) {
    if let Some(current_action) = state.0.actions.first() {
        if let Some((entity, character, animation, source_global_transform, source_transform)) =
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
                                target: Transform::from_translation(target_transform),
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
                    commands
                        .entity(entity)
                        .insert(animation.get_transition("die").unwrap());
                }
            }
            debug!("Changing state to PlayAnimation");
            combat_state.set(GameCombatState::PlayAnimation);
        } else {
            warn!("No character found for {:?}", current_action);
        }
    } else {
        combat_state.set(GameCombatState::WaitingForShop);
    }
}

fn animation_finished(
    mut battle: ResMut<BattleRes>,
    q_board_character: Query<(Entity, &BoardCharacter)>,
    mut ev_animation_finished: EventReader<AnimationFinished>,
    mut ev_board_change: EventWriter<BattleBoardChangedEvent>,
) {
    if let Some(current_action) = battle.0.actions.first().cloned() {
        for ev in ev_animation_finished.iter() {
            debug!("Animation finished for {:?}", ev.0);
            if let Some(character) = q_board_character.iter().find(|(entity, board_character)| {
                *entity == ev.0 && board_character.1.id == current_action.source
            }) {
                debug!("Animation finished for {:?}", character);
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
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_shop_change.iter() {
        debug!("Generating board");
        let character_handle = asset_server.load("textures/ui/character_fallback.png");
        let character_atlas =
            TextureAtlas::from_grid(character_handle, Vec2::new(64.0, 64.0), 14, 1, None, None);
        let character_atlas_handle = texture_atlases.add(character_atlas);

        let character_animation = animation::simple(0, 0)
            .with_state(
                AnimationState::new("die", AnimationIndices::new(1, 13))
                    .with_repeat_type(AnimationRepeatType::Once),
            )
            .with_global_transition(AnimationTransition {
                name: "die".to_string(),
                to_state: "die".to_string(),
                transition_type: AnimationTransitionType::Imediate,
            });

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
                    SpriteSheetBundle {
                        texture_atlas: character_atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_translation(Vec3::new(
                            68.0 * 2.0 * (idx % 4) as f32 + if idx < 4 { 0.0 } else { 68.0 } as f32,
                            if idx < 4 {
                                0.0
                            } else {
                                -1.0 * player_idx as f32 * -136.0
                            },
                            1.0,
                        )),
                        ..Default::default()
                    },
                    character_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    BoardCharacter(idx as u8, character.clone()),
                ));
            });
        }
        combat_state.set(GameCombatState::AnimationFinished);
    }
}