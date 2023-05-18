use crate::{
    model::game_users::GameUser, schema::game_users, service::character_service::get_board,
    Database,
};
use diesel::prelude::*;
use protocol::protocol::{BattleAction, BattleActionType, BattleResponse};
use rand::seq::IteratorRandom;

pub fn get_pairing(round: i32, mut players: Vec<GameUser>) -> Vec<(GameUser, GameUser)> {
    players.sort_by_key(|p| p.id);

    let mut active_players = players.iter().filter(|p| p.health > 0).collect::<Vec<_>>();
    if active_players.len() % 2 == 1 {
        active_players.push(players.iter().find(|p| p.health <= 0).unwrap());
    }

    let mut pairings = shifted(round, active_players.len() as i32);

    pairings.0.insert(0, 0);

    pairings
        .0
        .iter()
        .zip(pairings.1.iter())
        .map(|(a, b)| {
            (
                active_players[*a as usize].clone(),
                active_players[*b as usize].clone(),
            )
        })
        .collect::<Vec<_>>()
}

fn shifted(round: i32, players: i32) -> (Vec<i32>, Vec<i32>) {
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

pub async fn calculate_combat(db: &Database, players: &(GameUser, GameUser)) -> BattleResponse {
    let mut player_a_board = get_board(db, players.0.id).await.unwrap()[0..7].to_vec();
    let mut player_b_board = get_board(db, players.1.id).await.unwrap()[0..7].to_vec();
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
        // Get current player
        let (index, board, op_board) = if current_player {
            (
                &mut player_a_index,
                &mut player_a_board,
                &mut player_b_board,
            )
        } else {
            (
                &mut player_b_index,
                &mut player_b_board,
                &mut player_a_board,
            )
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
        let oponent = if op_board[0..4].iter().any(|c| c.is_some()) {
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

        // Calculate damage
        attacker.defense -= oponent.1.attack + oponent.1.attack_bonus;
        oponent.1.defense -= attacker.attack + attacker.attack_bonus;

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
        if attacker.defense <= 0 {
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
        }

        if oponent.1.defense <= 0 {
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
        }

        current_player = !current_player;
    }

    let player_a_survived = player_a_board.iter().filter(|c| c.is_some()).count();
    let player_b_survived = player_b_board.iter().filter(|c| c.is_some()).count();

    let (looser, dmg) = if player_a_survived > player_b_survived {
        (
            players.1.clone(),
            if player_b_survived == 0 {
                player_a_survived
            } else {
                0
            },
        )
    } else {
        (
            players.0.clone(),
            if player_a_survived == 0 {
                player_b_survived
            } else {
                0
            },
        )
    };

    db.run(move |con| {
        diesel::update(game_users::table)
            .set(game_users::health.eq(game_users::health - dmg as i32))
            .filter(game_users::id.eq(looser.id))
            .execute(con)
    })
    .await
    .unwrap();

    BattleResponse {
        actions,
        start_own,
        start_opponent,
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
    let players = vec![
        GameUser {
            id: 1,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 2,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 3,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 4,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 5,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 6,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 7,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 8,
            health: 1,
            ..Default::default()
        },
    ];

    assert_eq!(
        get_pairing(0, players.clone()),
        vec![
            (players[0].clone(), players[7].clone()),
            (players[1].clone(), players[6].clone()),
            (players[2].clone(), players[5].clone()),
            (players[3].clone(), players[4].clone()),
        ]
    );
    assert_eq!(
        get_pairing(1, players.clone()),
        vec![
            (players[0].clone(), players[1].clone()),
            (players[2].clone(), players[7].clone()),
            (players[3].clone(), players[6].clone()),
            (players[4].clone(), players[5].clone()),
        ]
    );
}

#[test]
fn test_pairing_with_dead_players() {
    let players = vec![
        GameUser {
            id: 1,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 2,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 3,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 4,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 5,
            health: 1,
            ..Default::default()
        },
        GameUser {
            id: 6,
            health: 0,
            ..Default::default()
        },
        GameUser {
            id: 7,
            health: 0,
            ..Default::default()
        },
        GameUser {
            id: 8,
            health: 0,
            ..Default::default()
        },
    ];

    assert_eq!(
        get_pairing(0, players.clone()),
        vec![
            (players[0].clone(), players[5].clone()),
            (players[1].clone(), players[4].clone()),
            (players[2].clone(), players[3].clone()),
        ]
    );

    assert_eq!(
        get_pairing(1, players.clone()),
        vec![
            (players[0].clone(), players[1].clone()),
            (players[2].clone(), players[5].clone()),
            (players[3].clone(), players[4].clone()),
        ]
    );
}
