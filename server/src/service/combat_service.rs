use crate::game::game_instance_player::GameInstancePlayer;
use protocol::protocol::{BattleAction, BattleActionType, BattleResponse};
use rand::seq::IteratorRandom;
use uuid::Uuid;

pub fn get_pairing(round: u16, mut players: Vec<&GameInstancePlayer>) -> Vec<(Uuid, Uuid)> {
    players.sort_by_key(|p| p.placement);

    let player_count = players.iter().filter(|p| p.placement.is_none()).count();
    let player_count = player_count + player_count % 2;

    let mut active_players = players[0..player_count].into_iter().collect::<Vec<_>>();

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
) -> BattleResponse {
    let mut player_a_board = players.0.board[0..7].to_vec();
    let mut player_b_board = players.1.board[0..7].to_vec();
    let start_own = player_a_board.clone();
    let start_opponent = player_b_board.clone();

    let mut current_player = rand::random::<bool>();
    let mut player_a_index = 0;
    let mut player_b_index = 0;

    let mut actions = vec![];

    // While there are still characters with attack on the board
    while player_a_board
        .iter()
        .any(|c| c.is_some() && c.as_ref().unwrap().attack + c.as_ref().unwrap().attack_bonus > 0)
        && player_b_board.iter().any(|c| {
            c.is_some() && c.as_ref().unwrap().attack + c.as_ref().unwrap().attack_bonus > 0
        })
    {
        execute_turn(
            current_player,
            &mut player_a_index,
            &mut player_a_board,
            &mut player_b_board,
            &mut player_b_index,
            &mut actions,
        );

        current_player = !current_player;
    }

    let player_a_survived = player_a_board.iter().filter(|c| c.is_some()).count();
    let player_b_survived = player_b_board.iter().filter(|c| c.is_some()).count();
    let player_a_lvl = players.0.get_lvl();
    let player_b_lvl = players.1.get_lvl();

    if player_a_survived > player_b_survived {
        players.1.health -= player_a_survived as i16 + player_a_lvl as i16;
    } else if player_b_survived > player_a_survived {
        players.0.health -= player_b_survived as i16 + player_b_lvl as i16;
    }

    BattleResponse {
        actions,
        start_own,
        start_opponent,
    }
}

fn execute_turn(
    current_player: bool,
    player_a_index: &mut usize,
    player_a_board: &mut Vec<Option<protocol::protocol::CharacterInstance>>,
    player_b_board: &mut Vec<Option<protocol::protocol::CharacterInstance>>,
    player_b_index: &mut usize,
    actions: &mut Vec<BattleAction>,
) {
    // Get current player
    let (index, board, op_board) = if current_player {
        (player_a_index, player_a_board, player_b_board)
    } else {
        (player_b_index, player_b_board, player_a_board)
    };

    // Get attacking creature
    while board[*index].is_none() {
        if *index < 6 {
            *index += 1;
        } else {
            *index = 0;
        }
    }
    let attacker = board[*index].as_mut().unwrap();

    // Get defending character
    let mut opponent_clone = op_board.clone();
    let mut oponent = if op_board[0..4].iter().any(|c| c.is_some()) {
        // Front row first
        opponent_clone[0..4]
            .iter_mut()
            .enumerate()
            .filter(|(_, c)| c.is_some())
            .map(|(i, c)| (i, c.as_mut().unwrap()))
            .choose(&mut rand::thread_rng())
            .unwrap()
    } else {
        // Back row otherwise
        opponent_clone[4..7]
            .iter_mut()
            .enumerate()
            .filter(|(_, c)| c.is_some())
            .map(|(i, c)| (i + 4, c.as_mut().unwrap()))
            .choose(&mut rand::thread_rng())
            .unwrap()
    };

    // Trigger on attack effects

    // Calculate damage
    perform_attack(attacker, &mut oponent);

    let attacker = attacker.clone();

    // Add battle action
    actions.push(BattleAction {
        action: BattleActionType::Attack,
        source: attacker.id,
        target: Some(oponent.1.id),
        result_own: if current_player {
            board.clone()
        } else {
            op_board.clone()
        },
        result_opponent: if current_player {
            op_board.clone()
        } else {
            board.clone()
        },
    });

    // Check for death
    if attacker.health + attacker.health_bonus <= 0 {
        board[*index] = None;

        // Add attacker death event
        actions.push(BattleAction {
            action: BattleActionType::Die,
            source: attacker.id,
            target: None,
            result_own: if current_player {
                board.clone()
            } else {
                op_board.clone()
            },
            result_opponent: if current_player {
                op_board.clone()
            } else {
                board.clone()
            },
        });
    } else {
        // Trigger on survive effects
    }

    if oponent.1.health + oponent.1.health_bonus <= 0 {
        op_board[oponent.0] = None;

        // Add defender death event
        actions.push(BattleAction {
            action: BattleActionType::Die,
            source: oponent.1.id,
            target: None,
            result_own: if current_player {
                board.clone()
            } else {
                op_board.clone()
            },
            result_opponent: if current_player {
                op_board.clone()
            } else {
                board.clone()
            },
        });
    } else {
        op_board[oponent.0] = Some(oponent.1.clone());

        // Trigger on survive effects
    }
}

fn perform_attack(
    attacker: &mut protocol::protocol::CharacterInstance,
    oponent: &mut (usize, &mut protocol::protocol::CharacterInstance),
) {
    attacker.health -= oponent.1.attack + oponent.1.attack_bonus;
    oponent.1.health -= attacker.attack + attacker.attack_bonus;
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
