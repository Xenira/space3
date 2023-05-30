use crate::{
    game::{game_instance::GameInstance, game_instance_player::GameInstancePlayer},
    model::{
        lobbies::Lobby,
        lobby_users::LobbyUser,
        polling::{ActivePolls, Channel},
    },
    schema::{lobbies, lobby_users},
    Database,
};
use diesel::{delete, prelude::*};
use futures::stream::{futures_unordered::FuturesUnordered, StreamExt};
use protocol::{
    gods::get_gods,
    protocol::{GameResult, Protocol},
};
use rand::seq::SliceRandom;
use rocket::log::private::debug;

pub async fn start_game(db: &Database, lobby: &Lobby) -> GameInstance {
    let lobby_id = lobby.id;
    let lobby = lobby.clone();
    let mut heros = get_gods().to_vec();
    heros.shuffle(&mut rand::thread_rng());

    // Get players from db
    let players = db
        .run(move |con| {
            let mut users: Vec<(Option<i32>, Option<String>)> = LobbyUser::belonging_to(&lobby)
                .select((lobby_users::user_id, lobby_users::display_name))
                .load::<(i32, String)>(con)
                .unwrap()
                .into_iter()
                .map(|(user, display_name)| (Some(user), Some(display_name)))
                .collect::<Vec<_>>();

            while users.len() < 8 {
                users.push((None, None));
            }

            users
        })
        .await;

    let players = players
        .into_iter()
        .map(|(user, display_name)| {
            let hero_choices = Vec::drain(&mut heros, 0..4).collect::<Vec<_>>();

            if let Some(display_name) = display_name {
                GameInstancePlayer::new(
                    user,
                    display_name,
                    hero_choices
                        .iter()
                        .map(|god| god.id)
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                )
            } else {
                let god = hero_choices.choose(&mut rand::thread_rng()).unwrap();
                GameInstancePlayer::new(
                    None,
                    format!("[BOT] {}", god.name),
                    hero_choices
                        .iter()
                        .map(|god| god.id)
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                )
                .with_god(god.clone())
            }
        })
        .collect::<Vec<_>>();

    let game = GameInstance::new(players.try_into().unwrap());

    db.run(move |con| delete(lobbies::table.filter(lobbies::id.eq(lobby_id))).execute(con))
        .await
        .unwrap();

    ActivePolls::join_users(
        Channel::Game(game.game_id),
        game.players
            .iter()
            .filter_map(|p| p.user_id)
            .collect::<Vec<_>>(),
    );
    notify_users(&game).await;

    for user in game.players.iter().filter(|p| p.user_id.is_some()) {
        ActivePolls::notify(
            user.user_id.unwrap(),
            Protocol::GameStartResponse(user.god_choices),
        )
        .await;
    }

    game
}

pub async fn next_turn(game: &mut GameInstance) -> bool {
    debug!("Next turn for game {:?}", game.game_id);

    let ended = game.next_turn().await;

    notify_users(&game).await;

    ended
}

pub async fn update_player_placements(game: &mut GameInstance) -> QueryResult<()> {
    debug!("Updating player placements for game {:?}", game.game_id);
    let game_id = game.game_id;

    let mut next_placement = game.players.iter().fold(9, |acc, user| {
        if let Some(placement) = user.placement {
            acc.min(placement)
        } else {
            acc
        }
    }) - 1;

    let mut users = game
        .players
        .iter_mut()
        .filter(|user| user.health <= 0 && user.placement.is_none())
        .collect::<Vec<_>>();

    if users.is_empty() {
        return Ok(());
    }

    debug!("Updating player placements for dead users {:?}", users);

    debug!("Next placement: {}", next_placement);

    users.sort_by_key(|user| user.health);

    for user in users.iter_mut() {
        user.placement = Some(next_placement);
        next_placement -= 1;
    }

    users
        .iter()
        .filter(|user| user.user_id.is_some())
        .map(|user| {
            let user_id = user.user_id.unwrap();
            ActivePolls::leave_channel(&Channel::Game(game_id), &user_id);
            ActivePolls::notify(
                user_id,
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

    if game.is_game_over() {
        let winner = game.players.iter().find(|user| user.health > 0).unwrap();

        if winner.user_id.is_some() {
            ActivePolls::notify(
                winner.user_id.unwrap(),
                Protocol::GameEndResponse(GameResult {
                    place: 1,
                    reward: 100,
                    ranking: 1,
                }),
            )
            .await;
        }
        ActivePolls::close_channel(&Channel::Game(game_id));
    }

    QueryResult::Ok(())
}

pub async fn notify_users(game: &GameInstance) {
    ActivePolls::notify_channel(
        &Channel::Game(game.game_id),
        Protocol::GameUpdateResponse(protocol::protocol::GameUpdate { turn: game.turn }),
    )
    .await;
}
