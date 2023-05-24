use crate::{
    model::{
        game::{Game, GameUpdate, NewGame},
        game_user_avatar_choices::NewGameUserAvatarChoice,
        game_users::{GameUser, NewGameUser},
        lobbies::Lobby,
        lobby_users::LobbyUser,
        polling::{ActivePolls, Channel},
    },
    schema::{
        game_user_avatar_choices::{self},
        game_users, games, lobby_users, shops,
    },
    service::combat_service::{calculate_combat, get_pairing},
    Database,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{
    delete,
    dsl::{count_star, min},
    insert_into,
    prelude::*,
    update,
};
use futures::stream::futures_unordered::FuturesUnordered;
use futures::stream::StreamExt;
use protocol::protocol::{GameResult, Protocol};
use rand::seq::SliceRandom;
use rocket::log::private::{debug, warn};

pub async fn start_game(db: &Database, lobby: &Lobby) {
    let lobby = lobby.clone();
    let game = db
        .run(move |con| {
            let new_game = insert_into(games::table)
                .values(NewGame::new())
                .returning(games::table::all_columns())
                .get_result::<Game>(con)
                .unwrap();

            let mut heros = protocol::gods::GODS.to_vec();
            heros.shuffle(&mut rand::thread_rng());

            (
                new_game.clone(),
                LobbyUser::belonging_to(&lobby)
                    .select((lobby_users::user_id, lobby_users::display_name))
                    .load::<(i32, String)>(con)
                    .unwrap()
                    .iter()
                    .map(|(user, display_name)| {
                        let game_user = insert_into(game_users::table)
                            .values(NewGameUser::from_parents(new_game.id, *user, display_name))
                            .returning(game_users::id)
                            .get_result::<i32>(con)
                            .unwrap();

                        let hero_choices = Vec::drain(&mut heros, 0..4).collect::<Vec<_>>();

                        insert_into(game_user_avatar_choices::table)
                            .values(
                                hero_choices
                                    .iter()
                                    .map(|hero| {
                                        NewGameUserAvatarChoice::from_parents(
                                            new_game.id,
                                            game_user,
                                            hero.id,
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .execute(con)
                            .unwrap();

                        (user.clone(), hero_choices)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .await;

    ActivePolls::join_users(
        Channel::Game(game.0.id),
        game.1
            .iter()
            .map(|(user, _)| user.clone())
            .collect::<Vec<_>>(),
    );
    notify_users(&game.0).await;

    for (user, hero_choices) in game.1 {
        ActivePolls::notify(&user, Protocol::GameStartResponse(hero_choices)).await;
    }
}

pub async fn next_turn(db: &Database, game: &Game) {
    debug!("Next turn for game {:?}", game);
    let game_id = game.id;

    // Battle Turn
    let next_turn = game.current_round + 1;
    let battle_time = if next_turn % 2 == 0 {
        let game = game.clone();
        let all_users = db
            .run(move |con| GameUser::belonging_to(&game).load::<GameUser>(con).unwrap())
            .await;

        let pairings = get_pairing(next_turn, &all_users);

        chrono::Utc::now().naive_utc()
            + chrono::Duration::seconds(
                (*pairings
                    .iter()
                    .map(|pairing| execute_combat(db, pairing))
                    .collect::<FuturesUnordered<_>>()
                    .collect::<Vec<_>>()
                    .await
                    .iter()
                    .fold(None, |curr, next| {
                        if let Some(curr) = curr {
                            if next > curr {
                                Some(next)
                            } else {
                                Some(curr)
                            }
                        } else {
                            Some(next)
                        }
                    })
                    .unwrap_or(&5) as f64
                    * 1.5) as i64,
            )
    } else {
        get_next_turn_time(next_turn)
    };

    let game_res = game.clone();
    let game = game.clone();
    if let Some(game) = db
        .run(move |con| {
            let active_users = GameUser::belonging_to(&game)
                .filter(game_users::health.gt(0))
                .load::<GameUser>(con)
                .unwrap_or(vec![]);

            if active_users.len() <= 1 && next_turn % 2 == 1 {
                return None;
            }

            update(game_users::table)
                .filter(game_users::game_id.eq(game.id))
                .set(game_users::experience.eq(game_users::experience + 1))
                .execute(con)
                .unwrap();

            let game_update = GameUpdate {
                current_round: Some(next_turn),
                next_battle: Some(battle_time),
            };

            debug!("Updating game {:?} with {:?}", game, game_update);
            update(games::table)
                .filter(games::id.eq(game.id))
                .set(game_update)
                .execute(con)
                .unwrap();

            // TODO: Remove extra debugging money
            update(game_users::table)
                .filter(game_users::game_id.eq(game.id))
                .set(game_users::credits.eq((next_turn + 3) / 2 + 20))
                .execute(con)
                .unwrap();

            if next_turn % 2 == 0 {
                delete(shops::table)
                    .filter(shops::game_id.eq(game.id).and(shops::locked.eq(false)))
                    .execute(con)
                    .unwrap();

                update(shops::table)
                    .filter(shops::game_id.eq(game.id))
                    .set(shops::locked.eq(false))
                    .execute(con)
                    .unwrap();
            }

            Some(games::table.find(game.id).first::<Game>(con).unwrap())
        })
        .await
    {
        if next_turn % 2 != 0 {
            update_player_placements(db, &game).await;
        }
        notify_users(&game).await;
    } else {
        debug!("Game {} is over", game_id);
        update_player_placements(db, &game_res).await;
        let winning_player = db
            .run(move |con| {
                game_users::table
                    .filter(game_users::game_id.eq(game_id))
                    .order(game_users::health.desc())
                    .first::<GameUser>(con)
                    .unwrap()
            })
            .await;
        debug!("Winning player: {:?}", winning_player);
        close_game(db, game_id, &winning_player).await;
    }
}

fn get_next_turn_time(turn: i32) -> NaiveDateTime {
    let turn: i64 = turn.into();
    chrono::Utc::now().naive_utc() + chrono::Duration::seconds(90.min(30 + (turn / 2 - 1) * 5))
}

async fn execute_combat(db: &Database, pairing: &(GameUser, GameUser)) -> usize {
    let combat_result = calculate_combat(db, &pairing).await;
    let swapped_result = combat_result.swap_players();
    let action_len = combat_result.actions.len();

    if pairing.0.placement.is_some() {
        ActivePolls::notify(
            &pairing.0.user_id,
            Protocol::GameBattleResponse(combat_result),
        )
        .await;
    }

    if pairing.1.placement.is_some() {
        ActivePolls::notify(
            &pairing.1.user_id,
            Protocol::GameBattleResponse(swapped_result),
        )
        .await;
    }

    action_len
}

async fn update_player_placements(db: &Database, game: &Game) -> QueryResult<()> {
    debug!("Updating player placements for game {:?}", game);
    let game_id = game.id;
    let game = game.clone();
    let users = db
        .run(move |con| {
            let mut users = GameUser::belonging_to(&game)
                .filter(
                    game_users::health
                        .le(0)
                        .and(game_users::placement.is_null()),
                )
                .order(game_users::health.desc())
                .load::<GameUser>(con)?;

            if users.is_empty() {
                return QueryResult::Ok(users);
            }

            debug!("Updating player placements for dead users {:?}", users);

            let mut next_placement = GameUser::belonging_to(&game)
                .select((count_star(), min(game_users::placement)))
                .first::<(i64, Option<i32>)>(con)
                .map(|(count, min)| {
                    if let Some(min) = min {
                        min - 1
                    } else {
                        count as i32
                    }
                })?;

            debug!("Next placement: {}", next_placement);

            for user in users.iter_mut() {
                update(game_users::table)
                    .filter(game_users::id.eq(user.id))
                    .set(game_users::placement.eq(next_placement))
                    .execute(con)
                    .unwrap();
                user.placement = Some(next_placement);
                next_placement -= 1;
            }

            QueryResult::Ok(users)
        })
        .await;

    match users {
        Ok(users) => {
            users
                .iter()
                .map(|user: &GameUser| {
                    ActivePolls::notify(
                        &user.user_id,
                        Protocol::GameEndResponse(GameResult {
                            place: user.placement.unwrap(),
                            reward: 100,
                            ranking: 1,
                        }),
                    )
                })
                .collect::<FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await;
            for user in users {
                ActivePolls::leave_channel(&Channel::Game(game_id), &user.user_id);
            }

            Ok(())
        }
        Err(e) => {
            warn!("Error updating player placements: {:?}", e);
            Err(e)
        }
    }
}

pub async fn notify_users(game: &Game) {
    let game = game.clone();
    ActivePolls::notify_channel(
        &Channel::Game(game.id),
        Protocol::GameUpdateResponse(protocol::protocol::GameUpdate {
            turn: game.current_round,
            next_turn_at: game.next_battle.map(|time| DateTime::from_utc(time, Utc)),
        }),
    )
    .await;
}

async fn close_game(db: &Database, game_id: i32, winning_player: &GameUser) {
    debug!("Closing game {}", game_id);
    db.run(move |con| {
        delete(games::table)
            .filter(games::id.eq(game_id))
            .execute(con)
            .unwrap()
    })
    .await;

    // Notify users
    ActivePolls::notify(
        &winning_player.user_id,
        Protocol::GameEndResponse(GameResult {
            place: 1,
            reward: 100,
            ranking: 1,
        }),
    )
    .await;

    // Close channel
    ActivePolls::close_channel(&Channel::Game(game_id));
}
