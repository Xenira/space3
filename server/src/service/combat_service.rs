use crate::game::game_instance_player::GameInstancePlayer;
use protocol::{
    protocol::{BattleAction, BattleActionType, CharacterInstance},
    protocol_types::prelude::{Ability, AbilityTrigger, AbilityValue},
};
use rand::seq::IteratorRandom;
use rocket::log::private::debug;
use std::{cell::RefCell, cmp::Ordering, mem::swap, rc::Rc};
use uuid::Uuid;

#[derive(Debug)]
pub struct AbilityStackEntry {
    pub ability: Ability,
    pub target: Rc<RefCell<CharacterInstance>>,
    pub source: Rc<RefCell<CharacterInstance>>,
}

#[derive(Debug)]
pub struct Battle<'r> {
    pub board: &'r mut Vec<Option<Rc<RefCell<CharacterInstance>>>>,
    pub op_board: &'r mut Vec<Option<Rc<RefCell<CharacterInstance>>>>,
    pub current_player: &'r mut bool,
}

impl<'r> Battle<'r> {
    pub fn clone_board(&self) -> Vec<Option<CharacterInstance>> {
        self.board
            .iter()
            .map(|c| c.as_ref().map(|c| c.borrow().clone()))
            .collect()
    }

    pub fn clone_op_board(&self) -> Vec<Option<CharacterInstance>> {
        self.op_board
            .iter()
            .map(|c| c.as_ref().map(|c| c.borrow().clone()))
            .collect()
    }

    pub fn clone_player_a_board(&self) -> Vec<Option<CharacterInstance>> {
        if *self.current_player {
            self.clone_board()
        } else {
            self.clone_op_board()
        }
    }

    pub fn clone_player_b_board(&self) -> Vec<Option<CharacterInstance>> {
        if *self.current_player {
            self.clone_op_board()
        } else {
            self.clone_board()
        }
    }
}

pub fn get_pairing(round: u16, mut players: Vec<&GameInstancePlayer>) -> Vec<(Uuid, Uuid)> {
    players.sort_by_key(|p| p.placement);

    let player_count = players.iter().filter(|p| p.placement.is_none()).count();
    let player_count = player_count + player_count % 2;

    let mut active_players = players[0..player_count].iter().collect::<Vec<_>>();

    active_players.sort_by_key(|p| p.id);

    let mut pairings = shifted(round, active_players.len() as u16);

    pairings.0.insert(0, 0);

    pairings
        .0
        .into_iter()
        .zip(pairings.1.iter())
        .map(|(a, b)| {
            (
                active_players[a as usize].id,
                active_players[*b as usize].id,
            )
        })
        .collect::<Vec<_>>()
}

fn shifted(round: u16, players: u16) -> (Vec<i32>, Vec<i32>) {
    let group_a = 0..players / 2 - 1;
    let group_b = players / 2 - 1..players - 1;
    (
        group_a
            .map(|p| ((round + p) % (players - 1)) as i32 + 1)
            .collect::<Vec<_>>(),
        group_b
            .map(|p| ((round + p) % (players - 1)) as i32 + 1)
            .rev()
            .collect::<Vec<_>>(),
    )
}

pub async fn calculate_combat(
    players: &mut (&mut GameInstancePlayer, &mut GameInstancePlayer),
) -> (
    Vec<BattleAction>,
    Vec<Option<CharacterInstance>>,
    Vec<Option<CharacterInstance>>,
) {
    debug!("Calculating combat for {:?}", players);
    let start_own = players.0.board[0..7].to_vec();
    let start_opponent = players.1.board[0..7].to_vec();
    let player_a_board = &mut players.0.board[0..7]
        .iter()
        .map(|c| c.as_ref().map(|c| Rc::new(RefCell::new(c.clone()))))
        .collect::<Vec<_>>();
    let player_b_board = &mut players.1.board[0..7]
        .iter()
        .map(|c| c.as_ref().map(|c| Rc::new(RefCell::new(c.clone()))))
        .collect::<Vec<_>>();

    let current_player = &mut rand::random::<bool>();
    let mut player_a_index = 0;
    let mut player_b_index = 0;

    let mut actions = vec![];

    let (board, op_board) = if *current_player {
        (player_a_board, player_b_board)
    } else {
        (player_b_board, player_a_board)
    };
    let mut battle = Battle {
        board,
        op_board,
        current_player,
    };
    // While there are still characters with attack on the board
    while battle
        .board
        .iter()
        .filter_map(|c| c.as_ref())
        .any(|c| (*c).borrow().get_total_attack() > 0)
        && battle
            .op_board
            .iter()
            .filter_map(|c| c.as_ref())
            .any(|c| c.borrow().get_total_attack() > 0)
    {
        debug!("Calculating turn for {:?}", battle.current_player);

        execute_turn(
            &mut player_a_index,
            &mut player_b_index,
            &mut actions,
            &mut battle,
        );

        // Change current player
        swap(&mut battle.board, &mut battle.op_board);

        *battle.current_player = !*battle.current_player;
    }

    debug!("Calculating game result for {:?}", players);

    let player_a_survived = battle
        .clone_player_a_board()
        .iter()
        .filter(|c| c.is_some())
        .count();
    let player_b_survived = battle
        .clone_player_b_board()
        .iter()
        .filter(|c| c.is_some())
        .count();
    let player_a_lvl = players.0.get_lvl();
    let player_b_lvl = players.1.get_lvl();

    match player_a_survived.cmp(&player_b_survived) {
        Ordering::Greater => {
            players.1.health -= player_a_survived as i16 + player_a_lvl as i16;
        }
        Ordering::Less => {
            players.0.health -= player_b_survived as i16 + player_b_lvl as i16;
        }
        Ordering::Equal => (),
    }

    (actions, start_own, start_opponent)
}

fn execute_turn(
    player_a_index: &mut usize,
    player_b_index: &mut usize,
    actions: &mut Vec<BattleAction>,
    battle: &mut Battle,
) {
    // Get current player
    let index = if *battle.current_player {
        player_a_index
    } else {
        player_b_index
    };

    // Get attacking creature
    while battle.board[*index].is_none() {
        if *index < 6 {
            *index += 1;
        } else {
            *index = 0;
        }
    }
    let attacker = battle.board[*index].clone().unwrap();

    // Get defending character
    let mut opponent_clone = battle.op_board.clone();
    let oponent = if battle.op_board[0..4].iter().any(|c| c.is_some()) {
        // Front row first
        opponent_clone[0..4]
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_some())
            .map(|(i, c)| (i, c.clone().unwrap()))
            .choose(&mut rand::thread_rng())
            .unwrap()
    } else {
        // Back row otherwise
        opponent_clone[4..7]
            .iter_mut()
            .enumerate()
            .filter(|(_, c)| c.is_some())
            .map(|(i, c)| (i + 4, c.clone().unwrap()))
            .choose(&mut rand::thread_rng())
            .unwrap()
    };

    // Trigger on attack effects

    // Calculate damage
    perform_attack(attacker.clone(), oponent.clone());

    // Add battle action
    actions.push(BattleAction {
        action: BattleActionType::Attack,
        source: attacker.borrow().id,
        target: Some(oponent.1.borrow().id),
        result_own: battle.clone_player_a_board(),
        result_opponent: battle.clone_player_b_board(),
    });

    // Check for death
    if attacker.borrow().get_total_health() <= 0 {
        debug!("Character {:?} died", attacker.borrow().id);
        battle.board[*index] = None;

        // Add attacker death event
        actions.push(BattleAction {
            action: BattleActionType::Die,
            source: attacker.borrow().id,
            target: None,
            result_own: battle.clone_player_a_board(),
            result_opponent: battle.clone_player_b_board(),
        });
    } else {
        *index += 1;
    }

    if oponent.1.borrow().get_total_health() <= 0 {
        debug!("Character {:?} died", oponent.1.borrow().id);
        battle.op_board[oponent.0] = None;

        // Add defender death event
        actions.push(BattleAction {
            action: BattleActionType::Die,
            source: oponent.1.borrow().id,
            target: None,
            result_own: battle.clone_player_a_board(),
            result_opponent: battle.clone_player_b_board(),
        });
    }
}

fn perform_attack(
    attacker: Rc<RefCell<CharacterInstance>>,
    oponent: (usize, Rc<RefCell<CharacterInstance>>),
) {
    let mut stack = Vec::new();
    // On attack triggers
    for abilty in &attacker
        .borrow()
        .abilities
        .iter()
        .filter(|a| a.trigger == AbilityTrigger::OnAttack)
        .cloned()
        .collect::<Vec<_>>()
    {
        stack.push(AbilityStackEntry {
            ability: abilty.clone(),
            source: attacker.clone(),
            target: oponent.1.clone(),
        });
    }

    // On defend triggers
    for abilty in &oponent
        .1
        .borrow()
        .abilities
        .iter()
        .filter(|a| a.trigger == AbilityTrigger::OnDefend)
        .cloned()
        .collect::<Vec<_>>()
    {
        stack.push(AbilityStackEntry {
            ability: abilty.clone(),
            source: oponent.1.clone(),
            target: attacker.clone(),
        });
    }

    execute_stack_actions(&mut stack);

    stack.append(&mut damage(
        Some(attacker.clone()),
        oponent.1.clone(),
        attacker.borrow().get_total_attack(),
    ));
    stack.append(&mut damage(
        Some(oponent.1.clone()),
        attacker,
        oponent.1.borrow().get_total_attack(),
    ));

    execute_stack_actions(&mut stack);
}

fn damage(
    source: Option<Rc<RefCell<protocol::protocol::CharacterInstance>>>,
    target: Rc<RefCell<protocol::protocol::CharacterInstance>>,
    ammount: i32,
) -> Vec<AbilityStackEntry> {
    let mut result = Vec::new();

    // Apply damage
    target.borrow_mut().health -= ammount;

    // On survive/death triggers
    if let Some(source) = source.as_ref() {
        for ability in source
            .borrow()
            .abilities
            .iter()
            .filter(|a| {
                if target.borrow().get_total_health() > 0 {
                    a.trigger == AbilityTrigger::OnSurvive
                } else {
                    a.trigger == AbilityTrigger::OnDeath
                }
            })
            .cloned()
            .collect::<Vec<_>>()
        {
            result.push(AbilityStackEntry {
                ability: ability.clone(),
                source: target.clone(),
                target: source.clone(),
            });
        }
    }
    for abilty in &target
        .borrow()
        .abilities
        .iter()
        .filter(|a| {
            if target.borrow().get_total_health() > 0 {
                a.trigger == AbilityTrigger::OnSurvive
            } else {
                a.trigger == AbilityTrigger::OnDeath
            }
        })
        .cloned()
        .collect::<Vec<_>>()
    {
        result.push(AbilityStackEntry {
            ability: abilty.clone(),
            source: source.as_ref().unwrap_or(&target).clone(),
            target: target.clone(),
        });
    }

    result
}

fn execute_stack_actions(stack: &mut Vec<AbilityStackEntry>) {
    while let Some(entry) = stack.pop() {
        let targets = get_ability_targets(&entry);
        for target in targets {
            let mut result = apply_ability(&entry, target);
            stack.append(&mut result);
        }
    }
}

fn apply_ability(
    entry: &AbilityStackEntry,
    target: Rc<RefCell<CharacterInstance>>,
) -> Vec<AbilityStackEntry> {
    match &entry.ability.effect {
        protocol::protocol_types::prelude::AbilityEffect::Summon(_) => todo!(),
        protocol::protocol_types::prelude::AbilityEffect::Transform(_) => todo!(),
        protocol::protocol_types::prelude::AbilityEffect::Buff(attack, health, _) => {
            let attack_bonus = calculate_ammount(attack, entry);
            let health_bonus = calculate_ammount(health, entry);

            target.borrow_mut().attack_bonus += attack_bonus;
            target.borrow_mut().health_bonus += health_bonus;

            Vec::new()
        }
        protocol::protocol_types::prelude::AbilityEffect::Set(_, _) => todo!(),
        protocol::protocol_types::prelude::AbilityEffect::Damage(_) => todo!(),
        protocol::protocol_types::prelude::AbilityEffect::Slience(_) => todo!(),
        protocol::protocol_types::prelude::AbilityEffect::Stun(_) => todo!(),
        protocol::protocol_types::prelude::AbilityEffect::Stealth => todo!(),
        protocol::protocol_types::prelude::AbilityEffect::Taunt(_) => todo!(),
        protocol::protocol_types::prelude::AbilityEffect::Ranged => todo!(),
        protocol::protocol_types::prelude::AbilityEffect::Flying => Vec::new(),
        protocol::protocol_types::prelude::AbilityEffect::FirstStrike => Vec::new(),
    }
}

fn calculate_ammount(value: &AbilityValue, entry: &AbilityStackEntry) -> i32 {
    match value {
        AbilityValue::Plain(value) => *value,
        AbilityValue::PercentHealth(value) => {
            (entry.target.borrow().get_total_health() as f32 / 100.0 * *value as f32) as i32
        }
        AbilityValue::PercentAttack(value) => {
            (entry.target.borrow().get_total_attack() as f32 / 100.0 * *value as f32) as i32
        }
        AbilityValue::PercentMaxHealth(_) => todo!(),
        AbilityValue::PercentMaxAttack(_) => todo!(),
        AbilityValue::PercentTargetHealth(_) => todo!(),
        AbilityValue::PercentTargetAttack(_) => todo!(),
        AbilityValue::PercentTargetMaxHealth(_) => todo!(),
        AbilityValue::PercentTargetMaxAttack(_) => todo!(),
    }
}

fn get_ability_targets(ability: &AbilityStackEntry) -> Vec<Rc<RefCell<CharacterInstance>>> {
    match ability.ability.target {
        protocol::protocol_types::prelude::AbilityTarget::SelfTarget => {
            vec![ability.target.clone()]
        }
        protocol::protocol_types::prelude::AbilityTarget::EnemyTarget => todo!(),
        protocol::protocol_types::prelude::AbilityTarget::AllyTarget => todo!(),
        protocol::protocol_types::prelude::AbilityTarget::AllEnemyTarget => todo!(),
        protocol::protocol_types::prelude::AbilityTarget::AllAllyTarget => todo!(),
        protocol::protocol_types::prelude::AbilityTarget::AllTarget => todo!(),
    }
}

#[test]
fn test_shift() {
    assert_eq!(shifted(0, 8), (vec![1, 2, 3], vec![7, 6, 5, 4]));
    assert_eq!(shifted(1, 8), (vec![2, 3, 4], vec![1, 7, 6, 5]));
    assert_eq!(shifted(2, 8), (vec![3, 4, 5], vec![2, 1, 7, 6]));
    assert_eq!(shifted(3, 8), (vec![4, 5, 6], vec![3, 2, 1, 7]));
    assert_eq!(shifted(4, 8), (vec![5, 6, 7], vec![4, 3, 2, 1]));
    assert_eq!(shifted(5, 8), (vec![6, 7, 1], vec![5, 4, 3, 2]));
    assert_eq!(shifted(6, 8), (vec![7, 1, 2], vec![6, 5, 4, 3]));
    assert_eq!(shifted(7, 8), (vec![1, 2, 3], vec![7, 6, 5, 4]));

    assert_eq!(shifted(0, 6), (vec![1, 2], vec![5, 4, 3]));
    assert_eq!(shifted(1, 6), (vec![2, 3], vec![1, 5, 4]));
    assert_eq!(shifted(2, 6), (vec![3, 4], vec![2, 1, 5]));
    assert_eq!(shifted(3, 6), (vec![4, 5], vec![3, 2, 1]));
    assert_eq!(shifted(4, 6), (vec![5, 1], vec![4, 3, 2]));
    assert_eq!(shifted(5, 6), (vec![1, 2], vec![5, 4, 3]));

    assert_eq!(shifted(0, 4), (vec![1], vec![3, 2]));
    assert_eq!(shifted(1, 4), (vec![2], vec![1, 3]));
    assert_eq!(shifted(2, 4), (vec![3], vec![2, 1]));
    assert_eq!(shifted(3, 4), (vec![1], vec![3, 2]));
}

#[test]
fn test_pairing() {
    let mut players = vec![
        GameInstancePlayer {
            user_id: Some(1),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(2),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(3),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(4),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(5),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(6),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(7),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(8),
            health: 1,
            ..Default::default()
        },
    ];

    players.sort_by_key(|p| p.id);

    assert_eq!(
        get_pairing(0, players.iter().collect::<Vec<_>>()),
        vec![
            (players[0].id, players[7].id),
            (players[1].id, players[6].id),
            (players[2].id, players[5].id),
            (players[3].id, players[4].id),
        ]
    );
    assert_eq!(
        get_pairing(1, players.iter().collect::<Vec<_>>()),
        vec![
            (players[0].id, players[1].id),
            (players[2].id, players[7].id),
            (players[3].id, players[6].id),
            (players[4].id, players[5].id),
        ]
    );
}

#[test]
fn test_pairing_with_dead_players() {
    let mut players = vec![
        GameInstancePlayer {
            user_id: Some(1),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(2),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(3),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(4),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(5),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(6),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(7),
            health: 1,
            ..Default::default()
        },
        GameInstancePlayer {
            user_id: Some(8),
            health: 1,
            ..Default::default()
        },
    ];

    players.sort_by_key(|p| p.id);

    players[5].placement = Some(6);
    players[6].placement = Some(7);
    players[7].placement = Some(8);

    assert_eq!(
        get_pairing(0, players.iter().collect::<Vec<_>>()),
        vec![
            (players[0].id, players[5].id),
            (players[1].id, players[4].id),
            (players[2].id, players[3].id),
        ]
    );
    assert_eq!(
        get_pairing(1, players.iter().collect::<Vec<_>>()),
        vec![
            (players[0].id, players[1].id),
            (players[2].id, players[5].id),
            (players[3].id, players[4].id),
        ]
    );
}
